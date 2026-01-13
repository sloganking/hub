//! Process Manager - Start, stop, and monitor tool processes

use anyhow::{Context, Result};
use hub_common::{config, ToolConfig, ToolId, ToolStatus};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Manages child processes for all tools
#[derive(Debug)]
pub struct ProcessManager {
    /// Processes we spawned ourselves
    spawned_processes: HashMap<ToolId, Child>,
    /// External processes we detected (by PID)
    external_pids: HashMap<ToolId, u32>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            spawned_processes: HashMap::new(),
            external_pids: HashMap::new(),
        }
    }
    
    /// Initialize by detecting already-running tools (call after construction)
    pub fn init_detect_running(&mut self) {
        self.detect_running_tools();
    }

    /// Start a tool process with optional configuration
    pub fn start_tool(&mut self, tool_id: &ToolId) -> Result<()> {
        self.start_tool_with_config(tool_id, &ToolConfig::default())
    }

    /// Start a tool process with specific configuration
    pub fn start_tool_with_config(&mut self, tool_id: &ToolId, tool_config: &ToolConfig) -> Result<()> {
        // Check if already running (spawned by us)
        if let Some(child) = self.spawned_processes.get_mut(tool_id) {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process exited, we can restart
                    self.spawned_processes.remove(tool_id);
                }
                Ok(None) => {
                    // Still running
                    return Ok(());
                }
                Err(_) => {
                    // Error checking, remove and try to restart
                    self.spawned_processes.remove(tool_id);
                }
            }
        }
        
        // Check if running externally
        if let Some(pid) = self.external_pids.get(tool_id) {
            if is_process_running(*pid) {
                return Ok(()); // Already running externally
            } else {
                self.external_pids.remove(tool_id);
            }
        }

        // Find the binary
        let binary_path = self
            .find_binary(tool_id)
            .context(format!("Could not find binary for {}", tool_id.display_name()))?;

        println!("Starting {} from {:?}", tool_id.display_name(), binary_path);

        // Set up the command
        let mut cmd = Command::new(&binary_path);

        // Pass the API key via environment variable if available
        if tool_id.requires_api_key() {
            if let Ok(api_key) = config::load_api_key() {
                cmd.env("OPENAI_API_KEY", api_key);
            }
        }

        // Add hotkey arguments based on tool type
        self.add_hotkey_args(&mut cmd, tool_id, tool_config);

        // Hide console window for CLI tools on Windows
        #[cfg(windows)]
        {
            // CREATE_NO_WINDOW = 0x08000000
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            
            // Only hide window for CLI tools (GUI tools manage their own windows)
            match tool_id {
                ToolId::SpeakSelected | ToolId::QuickAssistant | ToolId::FlattenString | ToolId::OcrPaste => {
                    cmd.creation_flags(CREATE_NO_WINDOW);
                }
                _ => {}
            }
        }

        // Redirect stdin/stdout, but capture stderr for error reporting
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::piped());

        // Start the process
        let mut child = cmd
            .spawn()
            .context(format!("Failed to spawn {}", tool_id.display_name()))?;

        // Wait briefly to check if the process exits immediately with an error
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        match child.try_wait() {
            Ok(Some(exit_status)) => {
                // Process exited immediately - this is likely an error
                let mut stderr_output = String::new();
                if let Some(mut stderr) = child.stderr.take() {
                    use std::io::Read;
                    let _ = stderr.read_to_string(&mut stderr_output);
                }
                
                let error_msg = if !stderr_output.is_empty() {
                    stderr_output.lines().take(5).collect::<Vec<_>>().join("\n")
                } else {
                    format!("Process exited with code {:?}", exit_status.code())
                };
                
                return Err(anyhow::anyhow!("{}", error_msg));
            }
            Ok(None) => {
                // Process is still running - good! Drop stderr handle so it doesn't block
                drop(child.stderr.take());
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to check process status: {}", e));
            }
        }

        self.spawned_processes.insert(tool_id.clone(), child);

        Ok(())
    }

    /// Add hotkey command-line arguments based on tool type
    fn add_hotkey_args(&self, cmd: &mut Command, tool_id: &ToolId, tool_config: &ToolConfig) {
        // Skip for GUI apps that have their own config (desk-talk, typo-fix)
        // These tools read from their own Tauri config files
        match tool_id {
            ToolId::DeskTalk | ToolId::TypoFix => {
                // These have their own GUI config, don't pass CLI args
                return;
            }
            _ => {}
        }

        // For CLI tools, pass the hotkey as an argument
        if let Some(ref hotkey) = tool_config.hotkey {
            let arg_name = match tool_id {
                ToolId::SpeakSelected => "--ptt-key",
                ToolId::QuickAssistant => "--ptt-key",
                ToolId::FlattenString => "--trigger-key",
                ToolId::OcrPaste => "--trigger-key",
                _ => return,
            };
            cmd.arg(arg_name).arg(hotkey);
            println!("  Passing hotkey: {} {}", arg_name, hotkey);
        } else if let Some(special_key) = tool_config.special_hotkey {
            let arg_name = match tool_id {
                ToolId::SpeakSelected => "--special-ptt-key",
                ToolId::QuickAssistant => "--special-ptt-key",
                // flatten-string and ocr-paste don't support special keys
                _ => return,
            };
            cmd.arg(arg_name).arg(special_key.to_string());
            println!("  Passing special hotkey: {} {}", arg_name, special_key);
        } else {
            println!("  Warning: No hotkey configured for {}", tool_id.display_name());
        }
    }

    /// Stop a tool process (whether spawned by us or running externally)
    pub fn stop_tool(&mut self, tool_id: &ToolId) -> Result<()> {
        println!("Stopping {}...", tool_id.display_name());

        // First try to stop a process we spawned
        if let Some(mut child) = self.spawned_processes.remove(tool_id) {
            let pid = child.id();
            
            // Try graceful termination first
            #[cfg(windows)]
            {
                let _ = Command::new("taskkill")
                    .args(["/PID", &pid.to_string()])
                    .creation_flags(0x08000000) // CREATE_NO_WINDOW
                    .output();
            }

            std::thread::sleep(std::time::Duration::from_millis(500));

            match child.try_wait() {
                Ok(Some(_)) => {
                    println!("{} stopped gracefully", tool_id.display_name());
                }
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                    println!("{} force killed", tool_id.display_name());
                }
            }
            return Ok(());
        }

        // Try to stop an externally-started process
        if let Some(pid) = self.external_pids.remove(tool_id) {
            #[cfg(windows)]
            {
                let _ = Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/F"])
                    .creation_flags(0x08000000) // CREATE_NO_WINDOW
                    .output();
            }
            println!("{} (external, PID {}) stopped", tool_id.display_name(), pid);
            return Ok(());
        }

        // Not running
        Ok(())
    }

    /// Get the status of a tool
    pub fn get_status(&self, tool_id: &ToolId) -> ToolStatus {
        // Check spawned processes
        if self.spawned_processes.contains_key(tool_id) {
            return ToolStatus::Running;
        }
        
        // Check external processes
        if self.external_pids.contains_key(tool_id) {
            return ToolStatus::Running;
        }
        
        ToolStatus::Stopped
    }

    /// Update statuses by checking if processes are still running
    /// This is called frequently, so it must be FAST - no system calls for external processes
    pub fn refresh_statuses(&mut self) {
        // Check spawned processes - this is fast (just try_wait)
        let mut exited_spawned = Vec::new();
        for (tool_id, child) in self.spawned_processes.iter_mut() {
            match child.try_wait() {
                Ok(Some(_)) => {
                    exited_spawned.push(tool_id.clone());
                }
                Ok(None) => {
                    // Still running
                }
                Err(_) => {
                    exited_spawned.push(tool_id.clone());
                }
            }
        }
        for tool_id in exited_spawned {
            self.spawned_processes.remove(&tool_id);
        }
        
        // For external processes, we just trust they're still running
        // They'll be removed when we try to stop them or on next full scan
        // This avoids expensive tasklist calls every 2 seconds
    }
    
    /// Full scan for external processes (expensive - only call occasionally)
    pub fn full_scan(&mut self) {
        let running = get_all_running_processes();
        
        // Check external processes
        let mut exited_external = Vec::new();
        for (tool_id, pid) in self.external_pids.iter() {
            let exe_name = if cfg!(windows) {
                format!("{}.exe", tool_id.binary_name())
            } else {
                tool_id.binary_name().to_string()
            };
            
            let still_running = running.get(&exe_name.to_lowercase())
                .map(|&p| p == *pid)
                .unwrap_or(false);
            
            if !still_running {
                exited_external.push(tool_id.clone());
            }
        }
        for tool_id in exited_external {
            self.external_pids.remove(&tool_id);
        }

        // Detect newly-started external processes
        for tool_id in ToolId::all() {
            if self.spawned_processes.contains_key(tool_id) || self.external_pids.contains_key(tool_id) {
                continue;
            }
            let exe_name = if cfg!(windows) {
                format!("{}.exe", tool_id.binary_name())
            } else {
                tool_id.binary_name().to_string()
            };
            if let Some(&pid) = running.get(&exe_name.to_lowercase()) {
                self.external_pids.insert(tool_id.clone(), pid);
            }
        }
    }
    
    /// Detect tools that are already running (started outside the hub)
    fn detect_running_tools(&mut self) {
        // Get all running processes in one call (efficient)
        let running = get_all_running_processes();
        
        for tool_id in ToolId::all() {
            // Skip if we already know about this tool
            if self.spawned_processes.contains_key(tool_id) || self.external_pids.contains_key(tool_id) {
                continue;
            }
            
            // Check if this tool is running
            let exe_name = if cfg!(windows) {
                format!("{}.exe", tool_id.binary_name())
            } else {
                tool_id.binary_name().to_string()
            };
            
            if let Some(&pid) = running.get(&exe_name.to_lowercase()) {
                println!("Detected already-running {}: PID {}", tool_id.display_name(), pid);
                self.external_pids.insert(tool_id.clone(), pid);
            }
        }
    }

    /// Find the binary path for a tool
    fn find_binary(&self, tool_id: &ToolId) -> Option<PathBuf> {
        let binary_name = if cfg!(windows) {
            format!("{}.exe", tool_id.binary_name())
        } else {
            tool_id.binary_name().to_string()
        };

        // Try to find relative to current executable (production layout)
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // Same directory as hub (for portable/dev installs)
                let path = exe_dir.join(&binary_name);
                if path.exists() {
                    return Some(path);
                }

                // In a 'tools' subdirectory (bundled install)
                let path = exe_dir.join("tools").join(&binary_name);
                if path.exists() {
                    return Some(path);
                }

                // Check Tauri resource path
                // In bundled apps, resources are in: <exe_dir>/resources/tools/
                let path = exe_dir.join("resources").join("tools").join(&binary_name);
                if path.exists() {
                    return Some(path);
                }
            }
        }

        // Try workspace target directories (for development)
        if let Ok(cwd) = std::env::current_dir() {
            let workspace_paths = [
                // Workspace target directory (cargo builds all workspace members here)
                cwd.join("target").join("release").join(&binary_name),
                cwd.join("target").join("debug").join(&binary_name),
                // From workspace root - submodule's own target
                cwd.join("tools").join(tool_id_to_folder(tool_id)).join("target").join("release").join(&binary_name),
                cwd.join("tools").join(tool_id_to_folder(tool_id)).join("target").join("debug").join(&binary_name),
                // From crates/hub-dashboard (when running with cargo run)
                cwd.join("..").join("..").join("target").join("release").join(&binary_name),
                cwd.join("..").join("..").join("target").join("debug").join(&binary_name),
                cwd.join("..").join("..").join("tools").join(tool_id_to_folder(tool_id)).join("target").join("release").join(&binary_name),
                cwd.join("..").join("..").join("tools").join(tool_id_to_folder(tool_id)).join("target").join("debug").join(&binary_name),
            ];

            for path in &workspace_paths {
                if path.exists() {
                    return Some(path.canonicalize().unwrap_or(path.clone()));
                }
            }
        }

        None
    }

    /// Stop all running tools (only those we spawned, not external ones)
    pub fn stop_all(&mut self) {
        let tool_ids: Vec<_> = self.spawned_processes.keys().cloned().collect();
        for tool_id in tool_ids {
            let _ = self.stop_tool(&tool_id);
        }
        // Note: We don't stop external processes on hub close
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        // Don't stop tools when hub exits - let them keep running
        // The user can stop them manually or they'll be detected on next hub launch
    }
}

fn tool_id_to_folder(tool_id: &ToolId) -> &'static str {
    match tool_id {
        ToolId::DeskTalk => "desk-talk",
        ToolId::SpeakSelected => "speak-selected",
        ToolId::QuickAssistant => "quick-assistant",
        ToolId::FlattenString => "flatten-string",
        ToolId::TypoFix => "typo-fix",
        ToolId::OcrPaste => "ocr-paste",
    }
}

/// Get all running processes as a map of name -> PID (efficient single call)
#[cfg(windows)]
fn get_all_running_processes() -> HashMap<String, u32> {
    let mut result = HashMap::new();
    
    // Use tasklist to get all processes in one call
    let output = match Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
    {
        Ok(o) => o,
        Err(_) => return result,
    };
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse CSV output: "image_name","pid","session_name","session_num","mem_usage"
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            let name = parts[0].trim_matches('"').to_lowercase();
            let pid_str = parts[1].trim_matches('"');
            if let Ok(pid) = pid_str.parse::<u32>() {
                result.insert(name, pid);
            }
        }
    }
    
    result
}

#[cfg(not(windows))]
fn get_all_running_processes() -> HashMap<String, u32> {
    let mut result = HashMap::new();
    
    // Use ps on Unix-like systems
    let output = match Command::new("ps")
        .args(["-eo", "comm,pid"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return result,
    };
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[0].to_lowercase();
            if let Ok(pid) = parts[1].parse::<u32>() {
                result.insert(name, pid);
            }
        }
    }
    
    result
}


/// Check if a process with the given PID is still running
#[cfg(windows)]
fn is_process_running(pid: u32) -> bool {
    let output = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output();
    
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            !stdout.trim().is_empty() && !stdout.contains("No tasks")
        }
        Err(_) => false,
    }
}

#[cfg(not(windows))]
fn is_process_running(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

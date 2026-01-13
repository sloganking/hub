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
    processes: HashMap<ToolId, ManagedProcess>,
}

#[derive(Debug)]
struct ManagedProcess {
    child: Child,
    status: ToolStatus,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }

    /// Start a tool process with optional configuration
    pub fn start_tool(&mut self, tool_id: &ToolId) -> Result<()> {
        self.start_tool_with_config(tool_id, &ToolConfig::default())
    }

    /// Start a tool process with specific configuration
    pub fn start_tool_with_config(&mut self, tool_id: &ToolId, tool_config: &ToolConfig) -> Result<()> {
        // Check if already running
        if let Some(proc) = self.processes.get_mut(tool_id) {
            // Check if process is still alive
            match proc.child.try_wait() {
                Ok(Some(_)) => {
                    // Process exited, we can restart
                }
                Ok(None) => {
                    // Still running
                    return Ok(());
                }
                Err(_) => {
                    // Error checking, try to restart
                }
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

        self.processes.insert(
            tool_id.clone(),
            ManagedProcess {
                child,
                status: ToolStatus::Running,
            },
        );

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

    /// Stop a tool process
    pub fn stop_tool(&mut self, tool_id: &ToolId) -> Result<()> {
        if let Some(mut proc) = self.processes.remove(tool_id) {
            println!("Stopping {}...", tool_id.display_name());

            // Try graceful termination first
            #[cfg(windows)]
            {
                // On Windows, we can use taskkill for graceful termination
                let pid = proc.child.id();
                let _ = Command::new("taskkill")
                    .args(["/PID", &pid.to_string()])
                    .output();
            }

            // Give it a moment to exit gracefully
            std::thread::sleep(std::time::Duration::from_millis(500));

            // Check if it exited
            match proc.child.try_wait() {
                Ok(Some(_)) => {
                    println!("{} stopped gracefully", tool_id.display_name());
                }
                _ => {
                    // Force kill
                    let _ = proc.child.kill();
                    let _ = proc.child.wait();
                    println!("{} force killed", tool_id.display_name());
                }
            }
        }

        Ok(())
    }

    /// Get the status of a tool
    pub fn get_status(&self, tool_id: &ToolId) -> ToolStatus {
        if let Some(proc) = self.processes.get(tool_id) {
            // We can't call try_wait on a shared reference, so just return the stored status
            proc.status.clone()
        } else {
            ToolStatus::Stopped
        }
    }

    /// Update statuses by checking if processes are still running
    pub fn refresh_statuses(&mut self) {
        let mut exited = Vec::new();

        for (tool_id, proc) in self.processes.iter_mut() {
            match proc.child.try_wait() {
                Ok(Some(status)) => {
                    if status.success() {
                        proc.status = ToolStatus::Stopped;
                    } else {
                        proc.status = ToolStatus::Error(format!("Exited with code {:?}", status.code()));
                    }
                    exited.push(tool_id.clone());
                }
                Ok(None) => {
                    proc.status = ToolStatus::Running;
                }
                Err(e) => {
                    proc.status = ToolStatus::Error(e.to_string());
                    exited.push(tool_id.clone());
                }
            }
        }

        // Remove exited processes
        for tool_id in exited {
            self.processes.remove(&tool_id);
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

    /// Stop all running tools
    pub fn stop_all(&mut self) {
        let tool_ids: Vec<_> = self.processes.keys().cloned().collect();
        for tool_id in tool_ids {
            let _ = self.stop_tool(&tool_id);
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        self.stop_all();
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

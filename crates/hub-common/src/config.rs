//! Central configuration management for the Hub suite

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
};

use crate::tools::ToolId;
use crate::hotkeys::RegisteredHotkey;

/// Main Hub configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubConfig {
    /// Whether to start Hub with Windows
    #[serde(default)]
    pub auto_start: bool,
    
    /// Whether to start minimized to tray
    #[serde(default)]
    pub start_minimized: bool,
    
    /// Dark mode preference
    #[serde(default)]
    pub dark_mode: bool,
    
    /// Per-tool configuration
    #[serde(default)]
    pub tools: HashMap<ToolId, ToolConfig>,
    
    /// Registered hotkeys for all tools
    #[serde(default)]
    pub hotkeys: Vec<RegisteredHotkey>,
}

impl Default for HubConfig {
    fn default() -> Self {
        Self {
            auto_start: false,
            start_minimized: false,
            dark_mode: false,
            tools: HashMap::new(),
            hotkeys: Vec::new(),
        }
    }
}

/// Per-tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Whether this tool is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Whether to auto-start this tool when Hub starts
    #[serde(default)]
    pub auto_start: bool,
    
    /// The hotkey/trigger key for this tool (as a string like "F13", "F14", etc.)
    #[serde(default)]
    pub hotkey: Option<String>,
    
    /// Special hotkey code (for keys not in the standard enum)
    #[serde(default)]
    pub special_hotkey: Option<u32>,
    
    /// Tool-specific settings (stored as JSON value for flexibility)
    #[serde(default)]
    pub settings: serde_json::Value,
}

fn default_true() -> bool {
    true
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_start: false,
            hotkey: None,
            special_hotkey: None,
            settings: serde_json::Value::Null,
        }
    }
}

impl HubConfig {
    /// Get the configuration directory path
    pub fn config_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "hub", "productivity-hub")
            .context("Failed to determine project directories")?;
        let config_dir = proj_dirs.config_dir().to_path_buf();
        fs::create_dir_all(&config_dir)?;
        Ok(config_dir)
    }

    /// Get the configuration file path
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.json"))
    }

    /// Load configuration from disk
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            let config: HubConfig = serde_json::from_str(&contents)
                .context("Failed to parse config file")?;
            Ok(config)
        } else {
            Ok(HubConfig::default())
        }
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let contents = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(&config_path, contents)
            .context("Failed to write config file")?;
        Ok(())
    }

    /// Get tool configuration, creating default if not exists
    pub fn get_tool_config(&self, tool_id: &ToolId) -> ToolConfig {
        self.tools.get(tool_id).cloned().unwrap_or_default()
    }

    /// Update tool configuration
    pub fn set_tool_config(&mut self, tool_id: ToolId, config: ToolConfig) {
        self.tools.insert(tool_id, config);
    }
}

// === API Key Management ===

const KEYRING_SERVICE: &str = "productivity-hub";
const KEYRING_USER: &str = "openai-api-key";

/// Load the shared OpenAI API key from secure storage
pub fn load_api_key() -> Result<String> {
    // Try keyring first
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER) {
        if let Ok(key) = entry.get_password() {
            return Ok(key);
        }
    }

    // Fallback to .env file in config directory
    load_api_key_from_env()
}

/// Save the shared OpenAI API key to secure storage
pub fn save_api_key(api_key: &str) -> Result<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .context("Failed to create keyring entry")?;
    entry.set_password(api_key)
        .context("Failed to save API key to keyring")?;
    
    // Also save to .env as backup
    let _ = save_api_key_to_env(api_key);
    
    Ok(())
}

/// Delete the shared OpenAI API key from secure storage
pub fn delete_api_key() -> Result<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .context("Failed to create keyring entry")?;
    entry.delete_credential()
        .context("Failed to delete API key from keyring")
}

/// Check if an API key is configured
pub fn has_api_key() -> bool {
    load_api_key().is_ok()
}

fn load_api_key_from_env() -> Result<String> {
    let config_dir = HubConfig::config_dir()?;
    let env_path = config_dir.join(".env");
    
    if env_path.exists() {
        let contents = fs::read_to_string(&env_path)?;
        for line in contents.lines() {
            if let Some(key) = line.strip_prefix("OPENAI_API_KEY=") {
                return Ok(key.to_string());
            }
        }
    }
    
    Err(anyhow::anyhow!("No API key found in keyring or .env file"))
}

fn save_api_key_to_env(api_key: &str) -> Result<()> {
    let config_dir = HubConfig::config_dir()?;
    let env_path = config_dir.join(".env");
    fs::write(&env_path, format!("OPENAI_API_KEY={}", api_key))
        .context("Failed to write .env file")?;
    Ok(())
}

// === Windows Auto-start ===

#[cfg(windows)]
pub fn enable_autostart() -> Result<()> {
    use std::env;
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    let exe_path = env::current_exe()?;
    let exe_str = exe_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid exe path"))?;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    let (key, _) = hkcu.create_subkey(path)?;
    key.set_value("ProductivityHub", &exe_str)?;
    Ok(())
}

#[cfg(windows)]
pub fn disable_autostart() -> Result<()> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    if let Ok(key) = hkcu.open_subkey(path) {
        let _ = key.delete_value("ProductivityHub");
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn enable_autostart() -> Result<()> {
    Err(anyhow::anyhow!("Auto-start not implemented for this platform"))
}

#[cfg(not(windows))]
pub fn disable_autostart() -> Result<()> {
    Ok(())
}

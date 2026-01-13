//! Tauri commands for the Hub Dashboard

use crate::AppState;
use hub_common::{config, HubConfig, ToolConfig, ToolId, ToolStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

/// Frontend-friendly config representation
#[derive(Debug, Serialize, Deserialize)]
pub struct FrontendConfig {
    pub auto_start: bool,
    pub start_minimized: bool,
    pub dark_mode: bool,
    pub tools: HashMap<String, FrontendToolConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrontendToolConfig {
    pub enabled: bool,
    pub auto_start: bool,
    #[serde(default)]
    pub hotkey: Option<String>,
    #[serde(default)]
    pub special_hotkey: Option<u32>,
}

impl From<HubConfig> for FrontendConfig {
    fn from(config: HubConfig) -> Self {
        let tools = config
            .tools
            .into_iter()
            .map(|(id, tc)| {
                let key = match id {
                    ToolId::DeskTalk => "desk-talk",
                    ToolId::SpeakSelected => "speak-selected",
                    ToolId::QuickAssistant => "quick-assistant",
                    ToolId::FlattenString => "flatten-string",
                    ToolId::TypoFix => "typo-fix",
                    ToolId::OcrPaste => "ocr-paste",
                };
                (
                    key.to_string(),
                    FrontendToolConfig {
                        enabled: tc.enabled,
                        auto_start: tc.auto_start,
                        hotkey: tc.hotkey,
                        special_hotkey: tc.special_hotkey,
                    },
                )
            })
            .collect();

        FrontendConfig {
            auto_start: config.auto_start,
            start_minimized: config.start_minimized,
            dark_mode: config.dark_mode,
            tools,
        }
    }
}

fn string_to_tool_id(s: &str) -> Option<ToolId> {
    match s {
        "desk-talk" => Some(ToolId::DeskTalk),
        "speak-selected" => Some(ToolId::SpeakSelected),
        "quick-assistant" => Some(ToolId::QuickAssistant),
        "flatten-string" => Some(ToolId::FlattenString),
        "typo-fix" => Some(ToolId::TypoFix),
        "ocr-paste" => Some(ToolId::OcrPaste),
        _ => None,
    }
}

#[tauri::command]
pub fn get_config(state: State<AppState>) -> FrontendConfig {
    state.config.read().clone().into()
}

#[tauri::command]
pub fn save_config(state: State<AppState>, config: FrontendConfig) -> Result<(), String> {
    let mut hub_config = state.config.write();

    hub_config.auto_start = config.auto_start;
    hub_config.start_minimized = config.start_minimized;
    hub_config.dark_mode = config.dark_mode;

    // Update tool configs
    for (key, tc) in config.tools {
        if let Some(tool_id) = string_to_tool_id(&key) {
            hub_config.set_tool_config(
                tool_id,
                ToolConfig {
                    enabled: tc.enabled,
                    auto_start: tc.auto_start,
                    hotkey: tc.hotkey,
                    special_hotkey: tc.special_hotkey,
                    settings: serde_json::Value::Null,
                },
            );
        }
    }

    // Handle auto-start with Windows
    #[cfg(windows)]
    {
        if config.auto_start {
            let _ = hub_common::config::enable_autostart();
        } else {
            let _ = hub_common::config::disable_autostart();
        }
    }

    hub_config.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn has_api_key() -> bool {
    config::has_api_key()
}

#[tauri::command]
pub fn get_api_key_masked() -> Option<String> {
    config::load_api_key().ok().map(|key| {
        if key.len() > 8 {
            format!("{}...{}", &key[..4], &key[key.len()-4..])
        } else {
            "••••••••".to_string()
        }
    })
}

#[tauri::command]
pub fn get_api_key() -> Result<String, String> {
    config::load_api_key().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_api_key(api_key: String) -> Result<(), String> {
    config::save_api_key(&api_key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_api_key() -> Result<(), String> {
    config::delete_api_key().map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct ApiKeyValidation {
    pub valid: bool,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn validate_api_key() -> ApiKeyValidation {
    // Load the API key
    let api_key = match config::load_api_key() {
        Ok(key) => key,
        Err(_) => {
            return ApiKeyValidation {
                valid: false,
                error: Some("No API key configured".to_string()),
            }
        }
    };

    // Basic validation - check format
    if !api_key.starts_with("sk-") {
        return ApiKeyValidation {
            valid: false,
            error: Some("API key should start with 'sk-'".to_string()),
        };
    }

    if api_key.len() < 20 {
        return ApiKeyValidation {
            valid: false,
            error: Some("API key seems too short".to_string()),
        };
    }

    // For full validation, we'd need to make an API call
    // For now, just check format
    ApiKeyValidation {
        valid: true,
        error: None,
    }
}

#[tauri::command]
pub fn get_tool_statuses(state: State<AppState>) -> HashMap<String, String> {
    // First refresh to check which processes are still alive
    {
        let mut pm = state.process_manager.write();
        pm.refresh_statuses();
    }
    
    let pm = state.process_manager.read();
    let mut statuses = HashMap::new();

    for tool_id in ToolId::all() {
        let status = pm.get_status(tool_id);
        let key = match tool_id {
            ToolId::DeskTalk => "desk-talk",
            ToolId::SpeakSelected => "speak-selected",
            ToolId::QuickAssistant => "quick-assistant",
            ToolId::FlattenString => "flatten-string",
            ToolId::TypoFix => "typo-fix",
            ToolId::OcrPaste => "ocr-paste",
        };
        let status_str = match status {
            ToolStatus::Stopped => "Stopped",
            ToolStatus::Starting => "Starting",
            ToolStatus::Running => "Running",
            ToolStatus::Error(_) => "Error",
        };
        statuses.insert(key.to_string(), status_str.to_string());
    }

    statuses
}

#[tauri::command]
pub fn start_tool(state: State<AppState>, tool_id: String) -> Result<(), String> {
    let tool = string_to_tool_id(&tool_id).ok_or("Unknown tool")?;
    
    // Get the tool's configuration (including hotkey)
    let tool_config = {
        let config = state.config.read();
        config.get_tool_config(&tool)
    };
    
    let mut pm = state.process_manager.write();
    pm.start_tool_with_config(&tool, &tool_config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_tool(state: State<AppState>, tool_id: String) -> Result<(), String> {
    let tool = string_to_tool_id(&tool_id).ok_or("Unknown tool")?;
    let mut pm = state.process_manager.write();
    pm.stop_tool(&tool).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_tool_settings(tool_id: String) -> Result<(), String> {
    // For Tauri-based tools (desk-talk, typo-fix), we could potentially
    // communicate with them to show their settings window.
    // For now, just return an error indicating this isn't implemented.
    Err(format!(
        "Settings window for {} is not available from Hub. Start the tool and access its tray icon.",
        tool_id
    ))
}

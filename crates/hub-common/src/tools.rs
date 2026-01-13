//! Tool registry for managing the suite of productivity tools

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unique identifier for each tool in the suite
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolId {
    DeskTalk,
    SpeakSelected,
    QuickAssistant,
    FlattenString,
    TypoFix,
    OcrPaste,
}

impl ToolId {
    /// Get the display name for the tool
    pub fn display_name(&self) -> &'static str {
        match self {
            ToolId::DeskTalk => "DeskTalk",
            ToolId::SpeakSelected => "Speak Selected",
            ToolId::QuickAssistant => "Quick Assistant",
            ToolId::FlattenString => "Flatten String",
            ToolId::TypoFix => "Typo Fix",
            ToolId::OcrPaste => "OCR Paste",
        }
    }

    /// Get a short description of the tool
    pub fn description(&self) -> &'static str {
        match self {
            ToolId::DeskTalk => "Voice-to-text transcription with push-to-talk",
            ToolId::SpeakSelected => "Read selected text aloud using AI",
            ToolId::QuickAssistant => "Voice-activated AI assistant",
            ToolId::FlattenString => "Flatten clipboard text (remove newlines)",
            ToolId::TypoFix => "Fix typos in selected text using AI",
            ToolId::OcrPaste => "OCR from clipboard images",
        }
    }

    /// Get the binary name for the tool
    pub fn binary_name(&self) -> &'static str {
        match self {
            ToolId::DeskTalk => "desk-talk",
            ToolId::SpeakSelected => "speak-selected",
            ToolId::QuickAssistant => "quick-assistant",
            ToolId::FlattenString => "strflatten",
            ToolId::TypoFix => "typo-fix",
            ToolId::OcrPaste => "ocrp",
        }
    }

    /// Check if this tool requires OpenAI API key
    pub fn requires_api_key(&self) -> bool {
        match self {
            ToolId::DeskTalk => true,
            ToolId::SpeakSelected => true,
            ToolId::QuickAssistant => true,
            ToolId::FlattenString => false,
            ToolId::TypoFix => true,
            ToolId::OcrPaste => true,
        }
    }

    /// Get all tool IDs
    pub fn all() -> &'static [ToolId] {
        &[
            ToolId::DeskTalk,
            ToolId::SpeakSelected,
            ToolId::QuickAssistant,
            ToolId::FlattenString,
            ToolId::TypoFix,
            ToolId::OcrPaste,
        ]
    }
}

/// Status of a running tool
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolStatus {
    /// Tool is not running
    Stopped,
    /// Tool is starting up
    Starting,
    /// Tool is running normally
    Running,
    /// Tool encountered an error
    Error(String),
}

impl Default for ToolStatus {
    fn default() -> Self {
        ToolStatus::Stopped
    }
}

/// Information about a tool's runtime state
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub id: ToolId,
    pub status: ToolStatus,
    pub process_id: Option<u32>,
    pub binary_path: Option<PathBuf>,
}

impl ToolInfo {
    pub fn new(id: ToolId) -> Self {
        Self {
            id,
            status: ToolStatus::Stopped,
            process_id: None,
            binary_path: None,
        }
    }
}

/// Registry for managing tool states
#[derive(Debug, Default)]
pub struct ToolRegistry {
    tools: Vec<ToolInfo>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let tools = ToolId::all()
            .iter()
            .map(|id| ToolInfo::new(id.clone()))
            .collect();
        Self { tools }
    }

    /// Get tool info by ID
    pub fn get(&self, id: &ToolId) -> Option<&ToolInfo> {
        self.tools.iter().find(|t| &t.id == id)
    }

    /// Get mutable tool info by ID
    pub fn get_mut(&mut self, id: &ToolId) -> Option<&mut ToolInfo> {
        self.tools.iter_mut().find(|t| &t.id == id)
    }

    /// Update tool status
    pub fn set_status(&mut self, id: &ToolId, status: ToolStatus) {
        if let Some(tool) = self.get_mut(id) {
            tool.status = status;
        }
    }

    /// Update tool process ID
    pub fn set_process_id(&mut self, id: &ToolId, pid: Option<u32>) {
        if let Some(tool) = self.get_mut(id) {
            tool.process_id = pid;
        }
    }

    /// Get all tools
    pub fn all(&self) -> &[ToolInfo] {
        &self.tools
    }

    /// Get all running tools
    pub fn running(&self) -> Vec<&ToolInfo> {
        self.tools
            .iter()
            .filter(|t| t.status == ToolStatus::Running)
            .collect()
    }

    /// Find the binary path for a tool
    pub fn find_binary(&self, id: &ToolId) -> Option<PathBuf> {
        // First check if we have a cached path
        if let Some(tool) = self.get(id) {
            if let Some(path) = &tool.binary_path {
                if path.exists() {
                    return Some(path.clone());
                }
            }
        }

        // Try to find the binary relative to the current executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let binary_name = if cfg!(windows) {
                    format!("{}.exe", id.binary_name())
                } else {
                    id.binary_name().to_string()
                };

                // Check in same directory
                let path = exe_dir.join(&binary_name);
                if path.exists() {
                    return Some(path);
                }

                // Check in tools subdirectory
                let path = exe_dir.join("tools").join(&binary_name);
                if path.exists() {
                    return Some(path);
                }
            }
        }

        None
    }
}

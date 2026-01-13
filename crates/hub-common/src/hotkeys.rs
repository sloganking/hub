//! Hotkey registry to manage and avoid conflicts across tools

use rdev::Key;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::tools::ToolId;

/// A registered hotkey with its owner tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredHotkey {
    /// The tool that owns this hotkey
    pub tool_id: ToolId,
    
    /// Human-readable name for the action
    pub action_name: String,
    
    /// The key that triggers this action
    pub key: HotkeyKey,
    
    /// Optional modifier keys (Ctrl, Alt, Shift, etc.)
    #[serde(default)]
    pub modifiers: Vec<HotkeyModifier>,
}

/// Wrapper around rdev::Key that can be serialized
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", content = "value")]
pub enum HotkeyKey {
    /// Standard key by name
    Named(NamedKey),
    /// Unknown key by scan code
    Unknown(u32),
}

/// Named keys that can be used as hotkeys
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NamedKey {
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,
    
    // Navigation
    Insert, Delete, Home, End, PageUp, PageDown,
    
    // Arrow keys
    UpArrow, DownArrow, LeftArrow, RightArrow,
    
    // Numpad
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    NumLock, NumpadDivide, NumpadMultiply, NumpadSubtract, NumpadAdd, NumpadEnter,
    
    // Special keys
    Escape, Tab, CapsLock, Space, Backspace, Return,
    PrintScreen, ScrollLock, Pause,
    
    // Media keys
    MediaPlayPause, MediaStop, MediaPrevious, MediaNext,
    VolumeUp, VolumeDown, VolumeMute,
}

/// Modifier keys
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HotkeyModifier {
    Ctrl,
    Alt,
    Shift,
    Meta, // Windows key
}

impl From<HotkeyKey> for Key {
    fn from(key: HotkeyKey) -> Self {
        match key {
            HotkeyKey::Named(named) => named.into(),
            HotkeyKey::Unknown(code) => Key::Unknown(code),
        }
    }
}

impl From<NamedKey> for Key {
    fn from(key: NamedKey) -> Self {
        match key {
            NamedKey::F1 => Key::F1,
            NamedKey::F2 => Key::F2,
            NamedKey::F3 => Key::F3,
            NamedKey::F4 => Key::F4,
            NamedKey::F5 => Key::F5,
            NamedKey::F6 => Key::F6,
            NamedKey::F7 => Key::F7,
            NamedKey::F8 => Key::F8,
            NamedKey::F9 => Key::F9,
            NamedKey::F10 => Key::F10,
            NamedKey::F11 => Key::F11,
            NamedKey::F12 => Key::F12,
            NamedKey::F13 => Key::Unknown(124), // F13-F24 are Unknown on Windows
            NamedKey::F14 => Key::Unknown(125),
            NamedKey::F15 => Key::Unknown(126),
            NamedKey::F16 => Key::Unknown(127),
            NamedKey::F17 => Key::Unknown(128),
            NamedKey::F18 => Key::Unknown(129),
            NamedKey::F19 => Key::Unknown(130),
            NamedKey::F20 => Key::Unknown(131),
            NamedKey::F21 => Key::Unknown(132),
            NamedKey::F22 => Key::Unknown(133),
            NamedKey::F23 => Key::Unknown(134),
            NamedKey::F24 => Key::Unknown(135),
            NamedKey::Insert => Key::Insert,
            NamedKey::Delete => Key::Delete,
            NamedKey::Home => Key::Home,
            NamedKey::End => Key::End,
            NamedKey::PageUp => Key::PageUp,
            NamedKey::PageDown => Key::PageDown,
            NamedKey::UpArrow => Key::UpArrow,
            NamedKey::DownArrow => Key::DownArrow,
            NamedKey::LeftArrow => Key::LeftArrow,
            NamedKey::RightArrow => Key::RightArrow,
            NamedKey::Num0 => Key::Num0,
            NamedKey::Num1 => Key::Num1,
            NamedKey::Num2 => Key::Num2,
            NamedKey::Num3 => Key::Num3,
            NamedKey::Num4 => Key::Num4,
            NamedKey::Num5 => Key::Num5,
            NamedKey::Num6 => Key::Num6,
            NamedKey::Num7 => Key::Num7,
            NamedKey::Num8 => Key::Num8,
            NamedKey::Num9 => Key::Num9,
            NamedKey::NumLock => Key::NumLock,
            NamedKey::NumpadDivide => Key::Unknown(111),
            NamedKey::NumpadMultiply => Key::Unknown(106),
            NamedKey::NumpadSubtract => Key::Unknown(109),
            NamedKey::NumpadAdd => Key::Unknown(107),
            NamedKey::NumpadEnter => Key::Unknown(13),
            NamedKey::Escape => Key::Escape,
            NamedKey::Tab => Key::Tab,
            NamedKey::CapsLock => Key::CapsLock,
            NamedKey::Space => Key::Space,
            NamedKey::Backspace => Key::Backspace,
            NamedKey::Return => Key::Return,
            NamedKey::PrintScreen => Key::PrintScreen,
            NamedKey::ScrollLock => Key::ScrollLock,
            NamedKey::Pause => Key::Pause,
            NamedKey::MediaPlayPause => Key::Unknown(179),
            NamedKey::MediaStop => Key::Unknown(178),
            NamedKey::MediaPrevious => Key::Unknown(177),
            NamedKey::MediaNext => Key::Unknown(176),
            NamedKey::VolumeUp => Key::Unknown(175),
            NamedKey::VolumeDown => Key::Unknown(174),
            NamedKey::VolumeMute => Key::Unknown(173),
        }
    }
}

impl TryFrom<Key> for HotkeyKey {
    type Error = ();
    
    fn try_from(key: Key) -> Result<Self, Self::Error> {
        match key {
            Key::F1 => Ok(HotkeyKey::Named(NamedKey::F1)),
            Key::F2 => Ok(HotkeyKey::Named(NamedKey::F2)),
            Key::F3 => Ok(HotkeyKey::Named(NamedKey::F3)),
            Key::F4 => Ok(HotkeyKey::Named(NamedKey::F4)),
            Key::F5 => Ok(HotkeyKey::Named(NamedKey::F5)),
            Key::F6 => Ok(HotkeyKey::Named(NamedKey::F6)),
            Key::F7 => Ok(HotkeyKey::Named(NamedKey::F7)),
            Key::F8 => Ok(HotkeyKey::Named(NamedKey::F8)),
            Key::F9 => Ok(HotkeyKey::Named(NamedKey::F9)),
            Key::F10 => Ok(HotkeyKey::Named(NamedKey::F10)),
            Key::F11 => Ok(HotkeyKey::Named(NamedKey::F11)),
            Key::F12 => Ok(HotkeyKey::Named(NamedKey::F12)),
            Key::Insert => Ok(HotkeyKey::Named(NamedKey::Insert)),
            Key::Delete => Ok(HotkeyKey::Named(NamedKey::Delete)),
            Key::Home => Ok(HotkeyKey::Named(NamedKey::Home)),
            Key::End => Ok(HotkeyKey::Named(NamedKey::End)),
            Key::PageUp => Ok(HotkeyKey::Named(NamedKey::PageUp)),
            Key::PageDown => Ok(HotkeyKey::Named(NamedKey::PageDown)),
            Key::UpArrow => Ok(HotkeyKey::Named(NamedKey::UpArrow)),
            Key::DownArrow => Ok(HotkeyKey::Named(NamedKey::DownArrow)),
            Key::LeftArrow => Ok(HotkeyKey::Named(NamedKey::LeftArrow)),
            Key::RightArrow => Ok(HotkeyKey::Named(NamedKey::RightArrow)),
            Key::Escape => Ok(HotkeyKey::Named(NamedKey::Escape)),
            Key::Tab => Ok(HotkeyKey::Named(NamedKey::Tab)),
            Key::CapsLock => Ok(HotkeyKey::Named(NamedKey::CapsLock)),
            Key::Space => Ok(HotkeyKey::Named(NamedKey::Space)),
            Key::Backspace => Ok(HotkeyKey::Named(NamedKey::Backspace)),
            Key::Return => Ok(HotkeyKey::Named(NamedKey::Return)),
            Key::PrintScreen => Ok(HotkeyKey::Named(NamedKey::PrintScreen)),
            Key::ScrollLock => Ok(HotkeyKey::Named(NamedKey::ScrollLock)),
            Key::Pause => Ok(HotkeyKey::Named(NamedKey::Pause)),
            Key::Num0 => Ok(HotkeyKey::Named(NamedKey::Num0)),
            Key::Num1 => Ok(HotkeyKey::Named(NamedKey::Num1)),
            Key::Num2 => Ok(HotkeyKey::Named(NamedKey::Num2)),
            Key::Num3 => Ok(HotkeyKey::Named(NamedKey::Num3)),
            Key::Num4 => Ok(HotkeyKey::Named(NamedKey::Num4)),
            Key::Num5 => Ok(HotkeyKey::Named(NamedKey::Num5)),
            Key::Num6 => Ok(HotkeyKey::Named(NamedKey::Num6)),
            Key::Num7 => Ok(HotkeyKey::Named(NamedKey::Num7)),
            Key::Num8 => Ok(HotkeyKey::Named(NamedKey::Num8)),
            Key::Num9 => Ok(HotkeyKey::Named(NamedKey::Num9)),
            Key::NumLock => Ok(HotkeyKey::Named(NamedKey::NumLock)),
            Key::Unknown(code) => Ok(HotkeyKey::Unknown(code)),
            _ => Err(()),
        }
    }
}

/// Registry for managing hotkeys across all tools
#[derive(Debug, Default)]
pub struct HotkeyRegistry {
    hotkeys: Vec<RegisteredHotkey>,
}

impl HotkeyRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Load registry from existing hotkeys
    pub fn from_hotkeys(hotkeys: Vec<RegisteredHotkey>) -> Self {
        Self { hotkeys }
    }

    /// Register a hotkey for a tool
    pub fn register(
        &mut self,
        tool_id: ToolId,
        action_name: String,
        key: HotkeyKey,
        modifiers: Vec<HotkeyModifier>,
    ) -> Result<(), HotkeyConflict> {
        // Check for conflicts
        if let Some(conflict) = self.find_conflict(&key, &modifiers) {
            return Err(HotkeyConflict {
                existing: conflict.clone(),
            });
        }

        self.hotkeys.push(RegisteredHotkey {
            tool_id,
            action_name,
            key,
            modifiers,
        });

        Ok(())
    }

    /// Unregister all hotkeys for a tool
    pub fn unregister_tool(&mut self, tool_id: &ToolId) {
        self.hotkeys.retain(|h| &h.tool_id != tool_id);
    }

    /// Unregister a specific hotkey
    pub fn unregister(&mut self, key: &HotkeyKey, modifiers: &[HotkeyModifier]) {
        self.hotkeys.retain(|h| &h.key != key || h.modifiers != modifiers);
    }

    /// Find a conflicting hotkey
    pub fn find_conflict(&self, key: &HotkeyKey, modifiers: &[HotkeyModifier]) -> Option<&RegisteredHotkey> {
        self.hotkeys.iter().find(|h| &h.key == key && h.modifiers == modifiers)
    }

    /// Get all registered hotkeys
    pub fn all(&self) -> &[RegisteredHotkey] {
        &self.hotkeys
    }

    /// Get hotkeys for a specific tool
    pub fn for_tool(&self, tool_id: &ToolId) -> Vec<&RegisteredHotkey> {
        self.hotkeys.iter().filter(|h| &h.tool_id == tool_id).collect()
    }

    /// Convert to a vec for serialization
    pub fn into_vec(self) -> Vec<RegisteredHotkey> {
        self.hotkeys
    }

    /// Get hotkeys grouped by tool
    pub fn by_tool(&self) -> HashMap<ToolId, Vec<&RegisteredHotkey>> {
        let mut map: HashMap<ToolId, Vec<&RegisteredHotkey>> = HashMap::new();
        for hotkey in &self.hotkeys {
            map.entry(hotkey.tool_id.clone())
                .or_default()
                .push(hotkey);
        }
        map
    }
}

/// Error when a hotkey conflicts with an existing registration
#[derive(Debug)]
pub struct HotkeyConflict {
    pub existing: RegisteredHotkey,
}

impl std::fmt::Display for HotkeyConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Hotkey already registered by {} for action '{}'",
            self.existing.tool_id.display_name(),
            self.existing.action_name
        )
    }
}

impl std::error::Error for HotkeyConflict {}

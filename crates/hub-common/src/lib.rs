//! Hub Common - Shared configuration and utilities for the Hub productivity suite
//!
//! This crate provides:
//! - Centralized configuration management
//! - Shared OpenAI API key storage
//! - Hotkey registry to avoid conflicts
//! - Tool registry for managing enabled tools

pub mod config;
pub mod hotkeys;
pub mod tools;

pub use config::{HubConfig, ToolConfig};
pub use hotkeys::{HotkeyRegistry, RegisteredHotkey};
pub use tools::{ToolId, ToolRegistry, ToolStatus};

/// Re-export rdev::Key for convenience
pub use rdev::Key;

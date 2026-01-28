//! License configuration storage

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use uuid::Uuid;

/// License plan types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LicensePlan {
    #[default]
    Monthly,
    Yearly,
    Lifetime,
}

impl std::fmt::Display for LicensePlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LicensePlan::Monthly => write!(f, "Monthly"),
            LicensePlan::Yearly => write!(f, "Yearly"),
            LicensePlan::Lifetime => write!(f, "Lifetime"),
        }
    }
}

/// Stored license configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicenseConfig {
    /// The license key (if activated)
    pub license_key: Option<String>,
    
    /// License plan type
    pub license_plan: Option<LicensePlan>,
    
    /// License status from LemonSqueezy (active, inactive, expired, disabled)
    pub license_status: Option<String>,
    
    /// LemonSqueezy instance ID for this machine
    pub instance_id: Option<String>,
    
    /// Machine ID (UUID, generated once per install)
    pub machine_id: String,
    
    /// Whether trial has been started (one-time per machine)
    #[serde(default)]
    pub trial_started: bool,
    
    /// Trial expiration timestamp (RFC3339)
    pub trial_expiration: Option<String>,
    
    /// Last successful validation timestamp
    pub last_validated: Option<String>,
    
    /// Customer email (from LemonSqueezy)
    pub customer_email: Option<String>,
}

impl LicenseConfig {
    /// Get the configuration directory path
    pub fn config_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "slking", "productivity-hub")
            .context("Failed to determine project directories")?;
        let config_dir = proj_dirs.config_dir().to_path_buf();
        fs::create_dir_all(&config_dir)?;
        Ok(config_dir)
    }

    /// Get the license config file path
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("license.json"))
    }

    /// Load configuration from disk
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)
                .context("Failed to read license config file")?;
            let mut config: LicenseConfig = serde_json::from_str(&contents)
                .context("Failed to parse license config file")?;
            
            // Ensure machine_id exists
            if config.machine_id.is_empty() {
                config.machine_id = Uuid::new_v4().to_string();
                config.save()?;
            }
            
            Ok(config)
        } else {
            // Create new config with fresh machine ID
            let config = LicenseConfig {
                machine_id: Uuid::new_v4().to_string(),
                ..Default::default()
            };
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let contents = serde_json::to_string_pretty(self)
            .context("Failed to serialize license config")?;
        fs::write(&config_path, contents)
            .context("Failed to write license config file")?;
        Ok(())
    }

    /// Clear license data (deactivate)
    pub fn clear_license(&mut self) -> Result<()> {
        self.license_key = None;
        self.license_plan = None;
        self.license_status = None;
        self.instance_id = None;
        self.last_validated = None;
        self.customer_email = None;
        self.save()
    }

    /// Get machine name for activation
    pub fn get_machine_name() -> String {
        hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".to_string())
    }
}

//! Hub Licensing - LemonSqueezy integration for Productivity Hub
//!
//! Provides license validation, activation, and trial management.

mod config;
pub mod lemonsqueezy;
pub mod trial;

pub use config::{LicenseConfig, LicensePlan};
pub use lemonsqueezy::{LemonSqueezyClient, LicenseInfo, LicenseStatus, ValidationResult, ActivationResult};
pub use trial::{TrialInfo, TrialStatus};

/// Check if the app is authorized to run (valid license OR active trial)
pub fn is_authorized() -> bool {
    let config = LicenseConfig::load().unwrap_or_default();
    
    // Check for valid license
    if config.license_key.is_some() && config.license_status == Some("active".to_string()) {
        return true;
    }
    
    // Check for active trial
    if let Some(trial_exp) = &config.trial_expiration {
        if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(trial_exp) {
            if exp > chrono::Utc::now() {
                return true;
            }
        }
    }
    
    false
}

/// Get the current authorization status with details
pub fn get_auth_status() -> AuthStatus {
    let config = LicenseConfig::load().unwrap_or_default();
    
    // Check for valid license
    if let Some(ref key) = config.license_key {
        if config.license_status == Some("active".to_string()) {
            return AuthStatus::Licensed {
                plan: config.license_plan.unwrap_or(LicensePlan::Monthly),
                key_preview: mask_license_key(key),
            };
        }
    }
    
    // Check for active trial
    if let Some(ref trial_exp) = config.trial_expiration {
        if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(trial_exp) {
            let now = chrono::Utc::now();
            if exp > now {
                let remaining = exp.signed_duration_since(now);
                return AuthStatus::Trial {
                    days_remaining: remaining.num_days() as u32,
                    hours_remaining: (remaining.num_hours() % 24) as u32,
                };
            } else {
                return AuthStatus::TrialExpired;
            }
        }
    }
    
    // Check if trial was already used
    if config.trial_started {
        return AuthStatus::TrialExpired;
    }
    
    AuthStatus::NoLicense
}

/// Authorization status enum
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum AuthStatus {
    /// User has a valid license
    Licensed {
        plan: LicensePlan,
        key_preview: String,
    },
    /// User is in trial period
    Trial {
        days_remaining: u32,
        hours_remaining: u32,
    },
    /// Trial has expired
    TrialExpired,
    /// No license and no trial started
    NoLicense,
}

impl AuthStatus {
    pub fn is_authorized(&self) -> bool {
        matches!(self, AuthStatus::Licensed { .. } | AuthStatus::Trial { .. })
    }
}

fn mask_license_key(key: &str) -> String {
    if key.len() > 8 {
        format!("{}...{}", &key[..4], &key[key.len()-4..])
    } else {
        "••••••••".to_string()
    }
}

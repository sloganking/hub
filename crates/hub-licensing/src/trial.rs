//! Trial system - 7 day free trial, one-time per machine

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::config::LicenseConfig;

/// Trial duration in days
pub const TRIAL_DAYS: i64 = 7;

/// Trial status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialInfo {
    /// Whether trial is currently active
    pub active: bool,
    /// Days remaining (0 if expired)
    pub days_remaining: u32,
    /// Hours remaining (0-23)
    pub hours_remaining: u32,
    /// Minutes remaining (0-59)
    pub minutes_remaining: u32,
    /// Expiration timestamp (RFC3339)
    pub expires_at: Option<String>,
    /// Whether trial was already used (can't start again)
    pub already_used: bool,
}

/// Trial status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrialStatus {
    /// Trial available but not started
    Available,
    /// Trial is active
    Active,
    /// Trial has expired
    Expired,
}

/// Get current trial status
pub fn get_trial_status() -> Result<TrialInfo> {
    let config = LicenseConfig::load()?;
    
    // Check if trial was never started
    if !config.trial_started {
        return Ok(TrialInfo {
            active: false,
            days_remaining: TRIAL_DAYS as u32,
            hours_remaining: 0,
            minutes_remaining: 0,
            expires_at: None,
            already_used: false,
        });
    }
    
    // Trial was started, check if still active
    if let Some(ref exp_str) = config.trial_expiration {
        let expiration = chrono::DateTime::parse_from_rfc3339(exp_str)
            .context("Failed to parse trial expiration")?;
        let now = Utc::now();
        
        if expiration > now {
            // Trial still active
            let remaining = expiration.signed_duration_since(now);
            return Ok(TrialInfo {
                active: true,
                days_remaining: remaining.num_days() as u32,
                hours_remaining: (remaining.num_hours() % 24) as u32,
                minutes_remaining: (remaining.num_minutes() % 60) as u32,
                expires_at: Some(exp_str.clone()),
                already_used: true,
            });
        }
    }
    
    // Trial expired
    Ok(TrialInfo {
        active: false,
        days_remaining: 0,
        hours_remaining: 0,
        minutes_remaining: 0,
        expires_at: config.trial_expiration,
        already_used: true,
    })
}

/// Start the trial (one-time per machine)
pub fn start_trial() -> Result<TrialInfo> {
    let mut config = LicenseConfig::load()?;
    
    // Check if trial was already used
    if config.trial_started {
        anyhow::bail!("Trial has already been used on this machine");
    }
    
    // Start trial
    let expiration = Utc::now() + Duration::days(TRIAL_DAYS);
    config.trial_started = true;
    config.trial_expiration = Some(expiration.to_rfc3339());
    config.save()?;
    
    Ok(TrialInfo {
        active: true,
        days_remaining: TRIAL_DAYS as u32,
        hours_remaining: 0,
        minutes_remaining: 0,
        expires_at: Some(expiration.to_rfc3339()),
        already_used: true,
    })
}

/// Format trial remaining time as a human-readable string
pub fn format_trial_remaining(info: &TrialInfo) -> String {
    if !info.active {
        if info.already_used {
            return "Trial expired".to_string();
        } else {
            return format!("{} day free trial available", TRIAL_DAYS);
        }
    }
    
    if info.days_remaining > 0 {
        format!("{} days, {} hours remaining", info.days_remaining, info.hours_remaining)
    } else if info.hours_remaining > 0 {
        format!("{} hours, {} minutes remaining", info.hours_remaining, info.minutes_remaining)
    } else {
        format!("{} minutes remaining", info.minutes_remaining)
    }
}

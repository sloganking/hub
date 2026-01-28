//! Tauri commands for license management

use hub_licensing::{
    AuthStatus, TrialInfo,
    lemonsqueezy, trial,
};
use serde::{Deserialize, Serialize};

/// Get current authorization status
#[tauri::command]
pub fn get_auth_status() -> AuthStatus {
    hub_licensing::get_auth_status()
}

/// Check if authorized (valid license or active trial)
#[tauri::command]
pub fn is_authorized() -> bool {
    hub_licensing::is_authorized()
}

/// Get trial information
#[tauri::command]
pub fn get_trial_info() -> Result<TrialInfo, String> {
    trial::get_trial_status().map_err(|e| e.to_string())
}

/// Start the 7-day trial
#[tauri::command]
pub fn start_trial() -> Result<TrialInfo, String> {
    trial::start_trial().map_err(|e| e.to_string())
}

/// Activate a license key
#[tauri::command]
pub async fn activate_license(license_key: String) -> Result<ActivationResultResponse, String> {
    let result = lemonsqueezy::activate_and_save(&license_key)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(ActivationResultResponse {
        success: result.activated,
        error: result.error,
        plan: result.meta.as_ref().map(|m| m.variant_name.clone()),
        customer_email: result.meta.as_ref().map(|m| m.customer_email.clone()),
    })
}

/// Validate existing license (refresh status from server)
#[tauri::command]
pub async fn validate_license() -> Result<ValidationResultResponse, String> {
    let result = lemonsqueezy::validate_existing()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(ValidationResultResponse {
        valid: result.valid,
        error: result.error,
        status: result.license_info.as_ref().map(|i| i.status.clone()),
    })
}

/// Deactivate license (remove from this machine)
#[tauri::command]
pub async fn deactivate_license() -> Result<bool, String> {
    lemonsqueezy::deactivate_and_clear()
        .await
        .map_err(|e| e.to_string())
}

/// Get the LemonSqueezy checkout URL for purchasing
#[tauri::command]
pub fn get_checkout_url(plan: String) -> String {
    match plan.as_str() {
        // Subscription (monthly/yearly variants are on the same checkout page)
        "monthly" | "yearly" => "https://slking.lemonsqueezy.com/checkout/buy/e84ca54b-c009-4262-a434-2528592e4077".to_string(),
        // Lifetime product
        "lifetime" => "https://slking.lemonsqueezy.com/checkout/buy/346b4776-f424-4c23-8980-227233e240cb".to_string(),
        // Fallback to store page
        _ => "https://slking.lemonsqueezy.com".to_string(),
    }
}

/// Open a checkout URL in the default browser
#[tauri::command]
pub fn open_checkout(plan: String) -> Result<(), String> {
    let url = match plan.as_str() {
        "monthly" | "yearly" => "https://slking.lemonsqueezy.com/checkout/buy/e84ca54b-c009-4262-a434-2528592e4077",
        "lifetime" => "https://slking.lemonsqueezy.com/checkout/buy/346b4776-f424-4c23-8980-227233e240cb",
        _ => "https://slking.lemonsqueezy.com",
    };
    
    // Use the Windows shell to open the URL
    #[cfg(windows)]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(not(windows))]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

// Response types for frontend

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivationResultResponse {
    pub success: bool,
    pub error: Option<String>,
    pub plan: Option<String>,
    pub customer_email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResultResponse {
    pub valid: bool,
    pub error: Option<String>,
    pub status: Option<String>,
}

//! LemonSqueezy API client for license validation and activation

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::{LicenseConfig, LicensePlan};

const API_BASE: &str = "https://api.lemonsqueezy.com/v1/licenses";

/// LemonSqueezy API client
pub struct LemonSqueezyClient {
    client: reqwest::Client,
}

impl LemonSqueezyClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Validate a license key
    pub async fn validate_license(&self, license_key: &str, instance_id: Option<&str>) -> Result<ValidationResult> {
        let mut form = vec![("license_key", license_key.to_string())];
        if let Some(id) = instance_id {
            form.push(("instance_id", id.to_string()));
        }

        let response = self.client
            .post(format!("{}/validate", API_BASE))
            .header("Accept", "application/json")
            .form(&form)
            .send()
            .await
            .context("Failed to connect to LemonSqueezy API")?;

        let result: ApiValidateResponse = response
            .json()
            .await
            .context("Failed to parse LemonSqueezy response")?;

        Ok(ValidationResult {
            valid: result.valid,
            error: result.error,
            license_info: result.license_key.map(|lk| LicenseInfo {
                id: lk.id,
                status: lk.status,
                key: lk.key,
                activation_limit: lk.activation_limit,
                activation_usage: lk.activation_usage,
                expires_at: lk.expires_at,
            }),
            instance_id: result.instance.map(|i| i.id),
            meta: result.meta.map(|m| LicenseMeta {
                store_id: m.store_id,
                product_id: m.product_id,
                product_name: m.product_name,
                variant_id: m.variant_id,
                variant_name: m.variant_name,
                customer_id: m.customer_id,
                customer_name: m.customer_name,
                customer_email: m.customer_email,
            }),
        })
    }

    /// Activate a license key on this machine
    pub async fn activate_license(&self, license_key: &str, instance_name: &str) -> Result<ActivationResult> {
        let form = vec![
            ("license_key", license_key.to_string()),
            ("instance_name", instance_name.to_string()),
        ];

        let response = self.client
            .post(format!("{}/activate", API_BASE))
            .header("Accept", "application/json")
            .form(&form)
            .send()
            .await
            .context("Failed to connect to LemonSqueezy API")?;

        let result: ApiActivateResponse = response
            .json()
            .await
            .context("Failed to parse LemonSqueezy activation response")?;

        Ok(ActivationResult {
            activated: result.activated,
            error: result.error,
            license_info: result.license_key.map(|lk| LicenseInfo {
                id: lk.id,
                status: lk.status,
                key: lk.key,
                activation_limit: lk.activation_limit,
                activation_usage: lk.activation_usage,
                expires_at: lk.expires_at,
            }),
            instance_id: result.instance.map(|i| i.id),
            meta: result.meta.map(|m| LicenseMeta {
                store_id: m.store_id,
                product_id: m.product_id,
                product_name: m.product_name,
                variant_id: m.variant_id,
                variant_name: m.variant_name,
                customer_id: m.customer_id,
                customer_name: m.customer_name,
                customer_email: m.customer_email,
            }),
        })
    }

    /// Deactivate a license key instance
    pub async fn deactivate_license(&self, license_key: &str, instance_id: &str) -> Result<bool> {
        let form = vec![
            ("license_key", license_key.to_string()),
            ("instance_id", instance_id.to_string()),
        ];

        let response = self.client
            .post(format!("{}/deactivate", API_BASE))
            .header("Accept", "application/json")
            .form(&form)
            .send()
            .await
            .context("Failed to connect to LemonSqueezy API")?;

        let result: ApiDeactivateResponse = response
            .json()
            .await
            .context("Failed to parse LemonSqueezy deactivation response")?;

        Ok(result.deactivated)
    }
}

impl Default for LemonSqueezyClient {
    fn default() -> Self {
        Self::new()
    }
}

// === Public types ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub error: Option<String>,
    pub license_info: Option<LicenseInfo>,
    pub instance_id: Option<String>,
    pub meta: Option<LicenseMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationResult {
    pub activated: bool,
    pub error: Option<String>,
    pub license_info: Option<LicenseInfo>,
    pub instance_id: Option<String>,
    pub meta: Option<LicenseMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub id: u64,
    pub status: String,
    pub key: String,
    pub activation_limit: Option<u32>,
    pub activation_usage: u32,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseMeta {
    pub store_id: u64,
    pub product_id: u64,
    pub product_name: String,
    pub variant_id: u64,
    pub variant_name: String,
    pub customer_id: u64,
    pub customer_name: String,
    pub customer_email: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LicenseStatus {
    Inactive,
    Active,
    Expired,
    Disabled,
}

// === API response types (internal) ===

#[derive(Debug, Deserialize)]
struct ApiValidateResponse {
    valid: bool,
    error: Option<String>,
    license_key: Option<ApiLicenseKey>,
    instance: Option<ApiInstance>,
    meta: Option<ApiMeta>,
}

#[derive(Debug, Deserialize)]
struct ApiActivateResponse {
    activated: bool,
    error: Option<String>,
    license_key: Option<ApiLicenseKey>,
    instance: Option<ApiInstance>,
    meta: Option<ApiMeta>,
}

#[derive(Debug, Deserialize)]
struct ApiDeactivateResponse {
    deactivated: bool,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiLicenseKey {
    id: u64,
    status: String,
    key: String,
    activation_limit: Option<u32>,
    activation_usage: u32,
    expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiInstance {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ApiMeta {
    store_id: u64,
    product_id: u64,
    product_name: String,
    variant_id: u64,
    variant_name: String,
    customer_id: u64,
    customer_name: String,
    customer_email: String,
}

// === Helper functions ===

/// Determine license plan from variant name
pub fn plan_from_variant_name(variant_name: &str) -> LicensePlan {
    let lower = variant_name.to_lowercase();
    if lower.contains("lifetime") {
        LicensePlan::Lifetime
    } else if lower.contains("year") {
        LicensePlan::Yearly
    } else {
        LicensePlan::Monthly
    }
}

/// Activate and save license to config
pub async fn activate_and_save(license_key: &str) -> Result<ActivationResult> {
    let client = LemonSqueezyClient::new();
    let machine_name = LicenseConfig::get_machine_name();
    
    let result = client.activate_license(license_key, &machine_name).await?;
    
    if result.activated {
        let mut config = LicenseConfig::load()?;
        config.license_key = Some(license_key.to_string());
        config.instance_id = result.instance_id.clone();
        
        if let Some(ref info) = result.license_info {
            config.license_status = Some(info.status.clone());
        }
        
        if let Some(ref meta) = result.meta {
            config.license_plan = Some(plan_from_variant_name(&meta.variant_name));
            config.customer_email = Some(meta.customer_email.clone());
        }
        
        config.last_validated = Some(chrono::Utc::now().to_rfc3339());
        config.save()?;
    }
    
    Ok(result)
}

/// Validate existing license (refresh status)
pub async fn validate_existing() -> Result<ValidationResult> {
    let config = LicenseConfig::load()?;
    
    let license_key = config.license_key
        .ok_or_else(|| anyhow::anyhow!("No license key configured"))?;
    
    let client = LemonSqueezyClient::new();
    let result = client.validate_license(&license_key, config.instance_id.as_deref()).await?;
    
    // Update config with fresh status
    if result.valid {
        let mut config = LicenseConfig::load()?;
        if let Some(ref info) = result.license_info {
            config.license_status = Some(info.status.clone());
        }
        config.last_validated = Some(chrono::Utc::now().to_rfc3339());
        config.save()?;
    }
    
    Ok(result)
}

/// Deactivate and clear license from config
pub async fn deactivate_and_clear() -> Result<bool> {
    let config = LicenseConfig::load()?;
    
    let license_key = config.license_key
        .ok_or_else(|| anyhow::anyhow!("No license key configured"))?;
    let instance_id = config.instance_id
        .ok_or_else(|| anyhow::anyhow!("No instance ID configured"))?;
    
    let client = LemonSqueezyClient::new();
    let deactivated = client.deactivate_license(&license_key, &instance_id).await?;
    
    if deactivated {
        let mut config = LicenseConfig::load()?;
        config.clear_license()?;
    }
    
    Ok(deactivated)
}

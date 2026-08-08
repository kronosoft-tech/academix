//! Subscription Cache — Desktop offline subscription status.
//!
//! Stores the subscription status as a JSON file in app_data_dir/academix/
//! so the desktop app can work offline with a 24-hour grace period.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Cached subscription status stored on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSubscriptionStatus {
    pub status: String,
    pub plan: Option<String>,
    pub checked_at: String,
}

/// Get the path to the subscription cache file.
/// Located at {data_local_dir}/academix/subscription_cache.json
pub fn get_cache_path() -> PathBuf {
    let app_data = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("academix");

    fs::create_dir_all(&app_data).ok();
    app_data.join("subscription_cache.json")
}

/// Read the cached subscription status from disk.
/// Returns None if the file doesn't exist or cannot be parsed.
pub fn read_cached_status(cache_path: &PathBuf) -> Option<CachedSubscriptionStatus> {
    let content = fs::read_to_string(cache_path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Write the subscription status to the cache file.
pub fn write_cached_status(
    cache_path: &PathBuf,
    status: &str,
    plan: Option<&str>,
) -> Result<(), String> {
    let cached = CachedSubscriptionStatus {
        status: status.to_string(),
        plan: plan.map(|p| p.to_string()),
        checked_at: Utc::now().to_rfc3339(),
    };

    let json = serde_json::to_string_pretty(&cached)
        .map_err(|e| format!("Failed to serialize subscription cache: {}", e))?;

    fs::write(cache_path, json)
        .map_err(|e| format!("Failed to write subscription cache: {}", e))?;

    Ok(())
}

/// Check if the cached status is still valid (less than 24 hours old).
pub fn is_cache_valid(checked_at: &str) -> bool {
    let parsed = match DateTime::parse_from_rfc3339(checked_at) {
        Ok(dt) => dt.with_timezone(&Utc),
        Err(_) => return false,
    };

    let now = Utc::now();
    let age = now.signed_duration_since(parsed);

    age.num_hours() < 24
}

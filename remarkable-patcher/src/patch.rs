#![allow(dead_code)]
//! Patch definitions and application

/// Result of checking a patch
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchStatus {
    Applied,
    NotApplied,
    Unknown,
}

/// Result of applying a patch
#[derive(Debug, Clone)]
pub enum PatchResult {
    Applied { offset: usize },
    AlreadyApplied,
    PatternNotFound,
}

/// Patch category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchCategory {
    DeveloperFeatures,
    Telemetry,
    Cosmetic,
    Subscription,
    Beta,
}

/// Risk level for patches
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Trait for all patches
pub trait Patch: Send + Sync {
    /// Unique name for the patch
    fn name(&self) -> &str;

    /// Human-readable description
    fn description(&self) -> &str;

    /// Category
    fn category(&self) -> PatchCategory;

    /// Risk level (if applicable)
    fn risk_level(&self) -> Option<RiskLevel> {
        Some(RiskLevel::Low)
    }

    /// Bytes to search for (original)
    fn before_bytes(&self) -> &[u8];

    /// Bytes to replace with (patched)
    fn after_bytes(&self) -> &[u8];

    /// Check if a name matches this patch
    fn matches_name(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        self.name().to_lowercase().contains(&name_lower)
    }

    /// Minimum supported version (optional)
    fn min_version(&self) -> Option<&str> {
        None
    }

    /// Maximum supported version (optional)
    fn max_version(&self) -> Option<&str> {
        None
    }
}

/// Enable InsecureSettings (developer/debug settings menu)
/// Pattern: Replaces a NOP sequence to skip an enable check
pub struct EnableInsecureSettings;

impl Patch for EnableInsecureSettings {
    fn name(&self) -> &str {
        "insecure-settings"
    }

    fn description(&self) -> &str {
        "Enable hidden InsecureSettings/developer settings menu. \
         Accessible via Settings > About > tap version 10 times."
    }

    fn category(&self) -> PatchCategory {
        PatchCategory::DeveloperFeatures
    }

    fn risk_level(&self) -> Option<RiskLevel> {
        Some(RiskLevel::Low)
    }

    fn before_bytes(&self) -> &[u8] {
        // CMP R0, #0; BEQ <skip> - disable check
        // Both are 2-byte thumb instructions = 4 bytes total
        b"\x00\x28\x00\xD0"
    }

    fn after_bytes(&self) -> &[u8] {
        // NOP; NOP - always enable
        b"\x00\xBF\x00\xBF"
    }
}

/// Disable telemetry reporting
pub struct DisableTelemetry;

impl Patch for DisableTelemetry {
    fn name(&self) -> &str {
        "disable-telemetry"
    }

    fn description(&self) -> &str {
        "Disable analytics and telemetry data collection. \
         Patches the telemetryv2 module initialization."
    }

    fn category(&self) -> PatchCategory {
        PatchCategory::Telemetry
    }

    fn risk_level(&self) -> Option<RiskLevel> {
        Some(RiskLevel::Low)
    }

    fn before_bytes(&self) -> &[u8] {
        // telemetryv2 module name (12 bytes including null)
        b"telemetryv2\x00"
    }

    fn after_bytes(&self) -> &[u8] {
        // Break module name so DI can't find it
        b"telemetrDIS\x00"
    }
}

/// Disable memfault crash reporting
pub struct DisableMemfault;

impl Patch for DisableMemfault {
    fn name(&self) -> &str {
        "disable-memfault"
    }

    fn description(&self) -> &str {
        "Disable Memfault crash/diagnostic reporting service."
    }

    fn category(&self) -> PatchCategory {
        PatchCategory::Telemetry
    }

    fn risk_level(&self) -> Option<RiskLevel> {
        Some(RiskLevel::Low)
    }

    fn before_bytes(&self) -> &[u8] {
        // memfault domain (8 bytes including null)
        b"memfault\x00"
    }

    fn after_bytes(&self) -> &[u8] {
        // Break domain
        b"memfaDIS\x00"
    }
}

/// Bypass subscription checks for local server
pub struct BypassSubscription;

impl Patch for BypassSubscription {
    fn name(&self) -> &str {
        "bypass-subscription"
    }

    fn description(&self) -> &str {
        "Bypass Connect subscription checks for local server usage. \
         Allows third-party cloud integrations without subscription."
    }

    fn category(&self) -> PatchCategory {
        PatchCategory::Subscription
    }

    fn risk_level(&self) -> Option<RiskLevel> {
        Some(RiskLevel::Medium)
    }

    fn before_bytes(&self) -> &[u8] {
        // subscriptionLevel string (18 bytes including null)
        b"subscriptionLevel\x00"
    }

    fn after_bytes(&self) -> &[u8] {
        // Break the check (change internal char to preserve length)
        b"subscriptionLev3l\x00"
    }
}

/// Enable beta features
pub struct EnableBetaFeatures;

impl Patch for EnableBetaFeatures {
    fn name(&self) -> &str {
        "enable-beta"
    }

    fn description(&self) -> &str {
        "Enable hidden beta/experimental features that are \
         normally gated behind beta enrollment."
    }

    fn category(&self) -> PatchCategory {
        PatchCategory::Beta
    }

    fn risk_level(&self) -> Option<RiskLevel> {
        Some(RiskLevel::Medium)
    }

    fn before_bytes(&self) -> &[u8] {
        // Experimental feature string (12 bytes including null)
        b"Experimental\x00"
    }

    fn after_bytes(&self) -> &[u8] {
        // Enable by default
        b"_Experimenal\x00"
    }
}

/// Skip developer password check
pub struct SkipDeveloperPassword;

impl Patch for SkipDeveloperPassword {
    fn name(&self) -> &str {
        "skip-dev-password"
    }

    fn description(&self) -> &str {
        "Skip the developer mode password requirement. \
         Allows direct access to developer settings."
    }

    fn category(&self) -> PatchCategory {
        PatchCategory::DeveloperFeatures
    }

    fn risk_level(&self) -> Option<RiskLevel> {
        Some(RiskLevel::Low)
    }

    fn before_bytes(&self) -> &[u8] {
        // developerPassword property (18 bytes including null)
        b"developerPassword\x00"
    }

    fn after_bytes(&self) -> &[u8] {
        // Break the property name (internal change)
        b"developerPassw0rd\x00"
    }
}

/// Redirect cloud sync to local server
pub struct RedirectCloudSync;

impl Patch for RedirectCloudSync {
    fn name(&self) -> &str {
        "redirect-cloud"
    }

    fn description(&self) -> &str {
        "Redirect cloud sync from my.remarkable.com to local server. \
         Requires running remarkable-server locally."
    }

    fn category(&self) -> PatchCategory {
        PatchCategory::Subscription
    }

    fn risk_level(&self) -> Option<RiskLevel> {
        Some(RiskLevel::High)
    }

    fn before_bytes(&self) -> &[u8] {
        // my.remarkable.com (18 bytes including null)
        b"my.remarkable.com\x00"
    }

    fn after_bytes(&self) -> &[u8] {
        // Redirect to localhost (same length)
        b"localhost:8080\x00\x00\x00\x00"
    }
}

/// Enable debug grid overlay
pub struct EnableDebugGrid;

impl Patch for EnableDebugGrid {
    fn name(&self) -> &str {
        "enable-debug-grid"
    }

    fn description(&self) -> &str {
        "Enable the debug grid overlay for UI development."
    }

    fn category(&self) -> PatchCategory {
        PatchCategory::DeveloperFeatures
    }

    fn risk_level(&self) -> Option<RiskLevel> {
        Some(RiskLevel::Low)
    }

    fn before_bytes(&self) -> &[u8] {
        // DebugGrid (10 bytes including null)
        b"DebugGrid\x00"
    }

    fn after_bytes(&self) -> &[u8] {
        // Break by changing internal char
        b"Debug_rid\x00"
    }
}

/// Disable OTA updates
pub struct DisableOtaUpdates;

impl Patch for DisableOtaUpdates {
    fn name(&self) -> &str {
        "disable-ota"
    }

    fn description(&self) -> &str {
        "Disable automatic OTA firmware updates. \
         Prevents device from auto-updating."
    }

    fn category(&self) -> PatchCategory {
        PatchCategory::Subscription
    }

    fn risk_level(&self) -> Option<RiskLevel> {
        Some(RiskLevel::Medium)
    }

    fn before_bytes(&self) -> &[u8] {
        // omahaupdate module (12 bytes including null)
        b"omahaupdate\x00"
    }

    fn after_bytes(&self) -> &[u8] {
        b"omahaupdatX\x00"
    }
}

/// Return all available patches
pub fn all_patches() -> Vec<Box<dyn Patch>> {
    vec![
        Box::new(EnableInsecureSettings),
        Box::new(DisableTelemetry),
        Box::new(DisableMemfault),
        Box::new(BypassSubscription),
        Box::new(EnableBetaFeatures),
        Box::new(SkipDeveloperPassword),
        Box::new(RedirectCloudSync),
        Box::new(EnableDebugGrid),
        Box::new(DisableOtaUpdates),
    ]
}

/// Get a patch by name
pub fn get_patch(name: &str) -> Option<Box<dyn Patch>> {
    all_patches().into_iter().find(|p| p.matches_name(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_patches_unique_names() {
        let patches = all_patches();
        let mut names: Vec<_> = patches.iter().map(|p| p.name()).collect();
        let original_len = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), original_len, "Duplicate patch names found");
    }

    #[test]
    fn test_patch_byte_lengths() {
        for patch in all_patches() {
            assert_eq!(
                patch.before_bytes().len(),
                patch.after_bytes().len(),
                "Patch '{}' has mismatched before/after lengths: {} vs {}",
                patch.name(),
                patch.before_bytes().len(),
                patch.after_bytes().len()
            );
        }
    }
}

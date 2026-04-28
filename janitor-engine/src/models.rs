use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Scan context provided to all scanners.
#[derive(Debug, Clone)]
pub struct ScanContext {
    /// Unique ID for this scan session.
    pub scan_id: String,

    /// Paths to scan. If empty, scanner should use defaults.
    pub target_paths: Vec<PathBuf>,

    /// Whether this scan requires elevation (admin).
    pub require_elevation: bool,

    /// Whether to include cloud-synced paths (OneDrive, Dropbox, iCloud).
    pub include_cloud_paths: bool,

    /// Whether to include developer caches (npm, cargo, etc) — opt-in.
    pub include_dev_caches: bool,
}

impl ScanContext {
    pub fn new() -> Self {
        Self {
            scan_id: Uuid::new_v4().to_string(),
            target_paths: Vec::new(),
            require_elevation: false,
            include_cloud_paths: false,
            include_dev_caches: false,
        }
    }
}

impl Default for ScanContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Risk level of a finding. Drives UI defaults and warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "low"),
            RiskLevel::Medium => write!(f, "medium"),
            RiskLevel::High => write!(f, "high"),
        }
    }
}

/// Category of finding. Groups related items in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    TempFiles,
    RecycleBin,
    BrowserCache,
    AppCache,
    WindowsUpdate,
    CrashDumps,
    Logs,
    ThumbnailCache,
    InstallerLeftovers,
    DownloadsOrphans,
    DevCache,
    EmptyDirs,
    Duplicates,
    Registry,
    Services,
    Tasks,
    Startup,
    Other,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Category::TempFiles => "Temp Files",
            Category::RecycleBin => "Recycle Bin",
            Category::BrowserCache => "Browser Cache",
            Category::AppCache => "App Cache",
            Category::WindowsUpdate => "Windows Update",
            Category::CrashDumps => "Crash Dumps",
            Category::Logs => "Logs",
            Category::ThumbnailCache => "Thumbnail Cache",
            Category::InstallerLeftovers => "Installer Leftovers",
            Category::DownloadsOrphans => "Downloads (Orphaned)",
            Category::DevCache => "Developer Cache",
            Category::EmptyDirs => "Empty Directories",
            Category::Duplicates => "Duplicates",
            Category::Registry => "Registry",
            Category::Services => "Services",
            Category::Tasks => "Scheduled Tasks",
            Category::Startup => "Startup Items",
            Category::Other => "Other",
        };
        write!(f, "{}", s)
    }
}

/// Target type: what kind of filesystem or system object is this?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetKind {
    File,
    Directory,
    RegistryKey,
    RegistryValue,
    Service,
    ScheduledTask,
}

impl std::fmt::Display for TargetKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetKind::File => write!(f, "File"),
            TargetKind::Directory => write!(f, "Directory"),
            TargetKind::RegistryKey => write!(f, "Registry Key"),
            TargetKind::RegistryValue => write!(f, "Registry Value"),
            TargetKind::Service => write!(f, "Service"),
            TargetKind::ScheduledTask => write!(f, "Scheduled Task"),
        }
    }
}

/// A single finding from a scanner. Immutable once created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Unique ID for this finding (within a scan).
    pub id: String,

    /// Which scanner produced this finding.
    pub scanner_id: String,

    /// Which rule matched (if any).
    pub rule_id: String,

    /// Category of finding.
    pub category: Category,

    /// Risk level.
    pub risk: RiskLevel,

    /// What are we talking about?
    pub target_kind: TargetKind,

    /// Path or registry key. String because registry keys aren't PathBuf.
    pub target_ref: String,

    /// Size in bytes (0 if N/A).
    pub size_bytes: u64,

    /// Age in days (0 if N/A).
    pub age_days: u32,

    /// Confidence [0.0, 1.0] that this is actually junk.
    pub confidence: f64,

    /// Detailed reason why this matched.
    pub reason: String,

    /// Suggested action.
    pub suggested_action: String,

    /// ISO 8601 timestamp when this finding was created.
    pub timestamp: String,
}

impl Finding {
    pub fn new(
        scanner_id: impl Into<String>,
        rule_id: impl Into<String>,
        category: Category,
        risk: RiskLevel,
        target_kind: TargetKind,
        target_ref: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            scanner_id: scanner_id.into(),
            rule_id: rule_id.into(),
            category,
            risk,
            target_kind,
            target_ref: target_ref.into(),
            size_bytes: 0,
            age_days: 0,
            confidence: 1.0,
            reason: String::new(),
            suggested_action: "quarantine".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.size_bytes = size;
        self
    }

    pub fn with_age(mut self, age: u32) -> Self {
        self.age_days = age;
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.suggested_action = action.into();
        self
    }
}

/// Result of a scan phase.
#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
    pub scan_id: String,
    pub findings: Vec<Finding>,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

impl ScanResult {
    pub fn new(scan_id: impl Into<String>) -> Self {
        Self {
            scan_id: scan_id.into(),
            findings: Vec::new(),
            errors: Vec::new(),
            duration_ms: 0,
        }
    }

    pub fn total_reclaimable_bytes(&self) -> u64 {
        self.findings.iter().map(|f| f.size_bytes).sum()
    }

    pub fn count_by_risk(&self, risk: RiskLevel) -> usize {
        self.findings.iter().filter(|f| f.risk == risk).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Finding builder ---

    #[test]
    fn finding_builder_works() {
        let f = Finding::new(
            "test_scanner",
            "rule_001",
            Category::TempFiles,
            RiskLevel::Low,
            TargetKind::File,
            "/tmp/test.txt",
        )
        .with_size(1024)
        .with_age(5)
        .with_reason("Old temp file")
        .with_confidence(0.95);

        assert_eq!(f.scanner_id, "test_scanner");
        assert_eq!(f.size_bytes, 1024);
        assert_eq!(f.age_days, 5);
        assert_eq!(f.confidence, 0.95);
        assert_eq!(f.reason, "Old temp file");
        assert_eq!(f.suggested_action, "quarantine"); // default
    }

    #[test]
    fn finding_with_action_overrides_default() {
        let f = Finding::new("s", "r", Category::RecycleBin, RiskLevel::Low, TargetKind::File, "p")
            .with_action("delete");
        assert_eq!(f.suggested_action, "delete");
    }

    #[test]
    fn finding_id_is_unique_per_instance() {
        let a = Finding::new("s", "r", Category::TempFiles, RiskLevel::Low, TargetKind::File, "p");
        let b = Finding::new("s", "r", Category::TempFiles, RiskLevel::Low, TargetKind::File, "p");
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn confidence_is_clamped() {
        let over = Finding::new("s", "r", Category::TempFiles, RiskLevel::Low, TargetKind::File, "p")
            .with_confidence(2.5);
        assert_eq!(over.confidence, 1.0);

        let under = Finding::new("s", "r", Category::TempFiles, RiskLevel::Low, TargetKind::File, "p")
            .with_confidence(-1.0);
        assert_eq!(under.confidence, 0.0);
    }

    #[test]
    fn finding_timestamp_is_rfc3339() {
        let f = Finding::new("s", "r", Category::TempFiles, RiskLevel::Low, TargetKind::File, "p");
        // Should parse as a valid RFC 3339 datetime
        chrono::DateTime::parse_from_rfc3339(&f.timestamp)
            .expect("timestamp must be valid RFC 3339");
    }

    // --- RiskLevel ---

    #[test]
    fn risk_level_ordering() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::Low < RiskLevel::High);
    }

    #[test]
    fn risk_level_display() {
        assert_eq!(RiskLevel::Low.to_string(), "low");
        assert_eq!(RiskLevel::Medium.to_string(), "medium");
        assert_eq!(RiskLevel::High.to_string(), "high");
    }

    // --- Category display ---

    #[test]
    fn category_display_not_empty() {
        let categories = [
            Category::TempFiles,
            Category::RecycleBin,
            Category::BrowserCache,
            Category::AppCache,
            Category::WindowsUpdate,
            Category::CrashDumps,
            Category::Logs,
            Category::ThumbnailCache,
            Category::InstallerLeftovers,
            Category::DownloadsOrphans,
            Category::DevCache,
            Category::EmptyDirs,
            Category::Duplicates,
            Category::Registry,
            Category::Services,
            Category::Tasks,
            Category::Startup,
            Category::Other,
        ];
        for cat in &categories {
            assert!(!cat.to_string().is_empty(), "{:?} display must not be empty", cat);
        }
    }

    // --- ScanContext ---

    #[test]
    fn scan_context_default_fields() {
        let ctx = ScanContext::new();
        assert!(!ctx.scan_id.is_empty());
        assert!(ctx.target_paths.is_empty());
        assert!(!ctx.require_elevation);
        assert!(!ctx.include_cloud_paths);
        assert!(!ctx.include_dev_caches);
    }

    #[test]
    fn scan_context_scan_ids_are_unique() {
        let a = ScanContext::new();
        let b = ScanContext::new();
        assert_ne!(a.scan_id, b.scan_id);
    }

    // --- ScanResult aggregation ---

    #[test]
    fn scan_result_aggregates() {
        let mut result = ScanResult::new("scan_123");
        result.findings.push(
            Finding::new("s1", "r1", Category::TempFiles, RiskLevel::Low, TargetKind::File, "/tmp/1")
                .with_size(100),
        );
        result.findings.push(
            Finding::new("s1", "r1", Category::TempFiles, RiskLevel::Low, TargetKind::File, "/tmp/2")
                .with_size(200),
        );

        assert_eq!(result.total_reclaimable_bytes(), 300);
        assert_eq!(result.count_by_risk(RiskLevel::Low), 2);
        assert_eq!(result.count_by_risk(RiskLevel::Medium), 0);
        assert_eq!(result.count_by_risk(RiskLevel::High), 0);
    }

    #[test]
    fn scan_result_mixed_risk_counts() {
        let mut result = ScanResult::new("scan_456");
        for risk in [RiskLevel::Low, RiskLevel::Medium, RiskLevel::High, RiskLevel::High] {
            result.findings.push(
                Finding::new("s", "r", Category::TempFiles, risk, TargetKind::File, "p"),
            );
        }
        assert_eq!(result.count_by_risk(RiskLevel::Low), 1);
        assert_eq!(result.count_by_risk(RiskLevel::Medium), 1);
        assert_eq!(result.count_by_risk(RiskLevel::High), 2);
    }

    #[test]
    fn scan_result_empty_is_zero() {
        let result = ScanResult::new("scan_empty");
        assert_eq!(result.total_reclaimable_bytes(), 0);
        assert_eq!(result.count_by_risk(RiskLevel::High), 0);
    }
}

use crate::{models::ScanContext, Finding, Result};

/// Core scanner trait. Every scanner implements this.
///
/// Scanners are independent, stateless, and run in parallel via rayon.
/// No shared mutable state between scanners.
pub trait Scanner: Send + Sync {
    /// Unique identifier for this scanner (e.g., "temp_dirs", "recycle_bin").
    fn id(&self) -> &'static str;

    /// Human-readable name.
    fn name(&self) -> &'static str;

    /// Whether this scanner requires admin/elevated privileges.
    fn requires_elevation(&self) -> bool {
        false
    }

    /// Main scan entry point.
    /// Returns a Vec of findings, or an error if the scan cannot proceed.
    ///
    /// Scanners should:
    /// - Call ctx blacklist checks via blacklist::is_path_safe() before yielding findings
    /// - Never hold state between calls
    /// - Return findings in order of confidence (high confidence first)
    fn scan(&self, ctx: &ScanContext) -> Result<Vec<Finding>>;

    /// Optional: returns true if this scanner is enabled by default.
    fn enabled_by_default(&self) -> bool {
        true
    }

    /// Optional: brief description of what this scanner does.
    fn description(&self) -> &'static str {
        "Scans for junk files."
    }
}

/// Builder for easy scanner registration.
pub struct ScannerBuilder;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Category, RiskLevel, TargetKind};

    struct MockScanner;

    impl Scanner for MockScanner {
        fn id(&self) -> &'static str {
            "mock"
        }

        fn name(&self) -> &'static str {
            "Mock Scanner"
        }

        fn scan(&self, _ctx: &ScanContext) -> Result<Vec<Finding>> {
            Ok(vec![Finding::new(
                "mock",
                "mock_rule",
                Category::TempFiles,
                RiskLevel::Low,
                TargetKind::File,
                "/tmp/mock.txt",
            )])
        }
    }

    #[test]
    fn scanner_trait_works() {
        let scanner = MockScanner;
        assert_eq!(scanner.id(), "mock");
        assert!(!scanner.requires_elevation());
        assert!(scanner.enabled_by_default());

        let ctx = ScanContext::new();
        let findings = scanner.scan(&ctx).unwrap();
        assert_eq!(findings.len(), 1);
    }
}

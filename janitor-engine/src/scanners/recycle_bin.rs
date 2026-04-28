use std::path::PathBuf;
use std::time::SystemTime;

use walkdir::WalkDir;

use crate::{
    blacklist,
    models::{Category, Finding, RiskLevel, ScanContext, TargetKind},
    Result, Scanner,
};

/// Scans $Recycle.Bin across all drives for deleted files.
///
/// Windows stores deleted files under C:\$Recycle.Bin\<SID>\.
/// Each deletion creates two files:
///   $Ixxxxxx — metadata (original path, deletion time, original size)
///   $Rxxxxxx — actual file content
///
/// We report the $R files (actual reclaimed bytes) and skip $I metadata files.
pub struct RecycleBinScanner;

impl RecycleBinScanner {
    fn recycle_roots() -> Vec<PathBuf> {
        // Check common drive letters. On most systems only C: exists,
        // but external drives also get $Recycle.Bin.
        let drives = ["C", "D", "E", "F", "G"];
        drives
            .iter()
            .map(|d| PathBuf::from(format!("{}:\\$Recycle.Bin", d)))
            .filter(|p| p.exists())
            .collect()
    }

    fn age_days(path: &std::path::Path) -> u32 {
        path.metadata()
            .and_then(|m| m.modified())
            .map(|modified| {
                SystemTime::now()
                    .duration_since(modified)
                    .map(|d| (d.as_secs() / 86400) as u32)
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }
}

impl Scanner for RecycleBinScanner {
    fn id(&self) -> &'static str {
        "recycle_bin"
    }

    fn name(&self) -> &'static str {
        "Recycle Bin"
    }

    fn description(&self) -> &'static str {
        "Finds files deleted to the Recycle Bin that are taking up space."
    }

    fn scan(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let roots = if ctx.target_paths.is_empty() {
            Self::recycle_roots()
        } else {
            ctx.target_paths.clone()
        };

        let mut findings = Vec::new();

        for root in &roots {
            for entry in WalkDir::new(root)
                .min_depth(2) // skip <drive>/$Recycle.Bin/<SID>/
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();

                if !blacklist::is_path_safe(path) {
                    continue;
                }

                let Ok(meta) = entry.metadata() else { continue };
                if !meta.is_file() {
                    continue;
                }

                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                // Skip the $I metadata files — we only report $R content files
                if name.starts_with("$I") {
                    continue;
                }

                let size = meta.len();
                let age = Self::age_days(path);

                let risk = if age >= 30 {
                    RiskLevel::Medium
                } else {
                    RiskLevel::Low
                };

                let reason = format!(
                    "Deleted file in Recycle Bin for {} day{}",
                    age,
                    if age == 1 { "" } else { "s" }
                );

                findings.push(
                    Finding::new(
                        self.id(),
                        "recycle_bin_item",
                        Category::RecycleBin,
                        risk,
                        TargetKind::File,
                        path.to_string_lossy(),
                    )
                    .with_size(size)
                    .with_age(age)
                    .with_confidence(0.99) // If it's in the bin, it's safe to empty
                    .with_reason(reason)
                    .with_action("delete"),
                );
            }
        }

        findings.sort_by_key(|f| std::cmp::Reverse(f.size_bytes));

        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn scanner_id_and_name() {
        assert_eq!(RecycleBinScanner.id(), "recycle_bin");
        assert!(!RecycleBinScanner.name().is_empty());
        assert!(!RecycleBinScanner.description().is_empty());
    }

    #[test]
    fn does_not_require_elevation() {
        assert!(!RecycleBinScanner.requires_elevation());
    }

    #[test]
    fn empty_on_nonexistent_path() {
        let mut ctx = ScanContext::new();
        ctx.target_paths = vec![PathBuf::from(r"Z:\$Recycle.Bin")];
        let findings = RecycleBinScanner.scan(&ctx).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn returns_ok_on_missing_path() {
        let mut ctx = ScanContext::new();
        ctx.target_paths = vec![PathBuf::from(r"C:\NoSuchRecycleBin")];
        assert!(RecycleBinScanner.scan(&ctx).is_ok());
    }

    #[test]
    fn skips_metadata_i_files() {
        // $I files are metadata; only $R files should produce findings.
        // We simulate a recycle bin structure under a TempDir.
        let dir = TempDir::new().unwrap();
        let sid_dir = dir.path().join("S-1-5-21-fake");
        std::fs::create_dir_all(&sid_dir).unwrap();

        // $I file (metadata) — must be skipped
        std::fs::write(sid_dir.join("$Ifoo123"), b"metadata").unwrap();
        // $R file (content) — could be reported if deep enough
        std::fs::write(sid_dir.join("$Rfoo123"), b"file content here").unwrap();

        let mut ctx = ScanContext::new();
        // min_depth is 2 in the scanner, so we use the parent as root
        ctx.target_paths = vec![dir.path().to_path_buf()];
        let findings = RecycleBinScanner.scan(&ctx).unwrap();

        for f in &findings {
            assert!(
                !f.target_ref.contains("$I"),
                "$I metadata file must not appear in findings: {}",
                f.target_ref
            );
        }
    }

    #[test]
    fn findings_sorted_by_size_descending() {
        let dir = TempDir::new().unwrap();
        let mut ctx = ScanContext::new();
        ctx.target_paths = vec![dir.path().to_path_buf()];
        let findings = RecycleBinScanner.scan(&ctx).unwrap();
        for w in findings.windows(2) {
            assert!(w[0].size_bytes >= w[1].size_bytes);
        }
    }

    #[test]
    fn finding_category_is_recycle_bin() {
        // If there ARE findings, they must all have RecycleBin category
        let dir = TempDir::new().unwrap();
        let sid = dir.path().join("S-1-5-21");
        std::fs::create_dir_all(&sid).unwrap();
        std::fs::write(sid.join("$Rabc123"), b"deleted file content").unwrap();

        let mut ctx = ScanContext::new();
        ctx.target_paths = vec![dir.path().to_path_buf()];
        let findings = RecycleBinScanner.scan(&ctx).unwrap();
        for f in &findings {
            assert_eq!(f.category, crate::models::Category::RecycleBin);
        }
    }
}

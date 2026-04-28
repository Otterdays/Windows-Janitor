use std::path::PathBuf;
use std::time::SystemTime;

use walkdir::WalkDir;

use crate::{
    blacklist,
    models::{Category, Finding, RiskLevel, ScanContext, TargetKind},
    Result, Scanner,
};

pub struct TempDirScanner;

impl TempDirScanner {
    fn temp_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // %TEMP% / %TMP%
        if let Ok(p) = std::env::var("TEMP") {
            paths.push(PathBuf::from(p));
        }
        if let Ok(p) = std::env::var("TMP") {
            let pb = PathBuf::from(&p);
            if !paths.contains(&pb) {
                paths.push(pb);
            }
        }

        // %LOCALAPPDATA%\Temp
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let p = PathBuf::from(local).join("Temp");
            if !paths.contains(&p) {
                paths.push(p);
            }
        }

        // C:\Windows\Temp (may need elevation for some files)
        let win_temp = PathBuf::from(r"C:\Windows\Temp");
        if !paths.contains(&win_temp) {
            paths.push(win_temp);
        }

        paths
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

impl Scanner for TempDirScanner {
    fn id(&self) -> &'static str {
        "temp_dirs"
    }

    fn name(&self) -> &'static str {
        "Temporary Files"
    }

    fn description(&self) -> &'static str {
        "Scans Windows temp directories for files that are safe to remove."
    }

    fn scan(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let roots = if ctx.target_paths.is_empty() {
            Self::temp_paths()
        } else {
            ctx.target_paths.clone()
        };

        let mut findings = Vec::new();

        for root in &roots {
            if !root.exists() {
                continue;
            }

            for entry in WalkDir::new(root)
                .min_depth(1)
                .max_depth(4)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();

                if !blacklist::is_path_safe(path) {
                    continue;
                }

                let Ok(meta) = entry.metadata() else { continue };

                // Only report files (not the directory itself)
                if !meta.is_file() {
                    continue;
                }

                let age = Self::age_days(path);
                let size = meta.len();

                // Skip brand-new files (< 1 day) — still in use
                if age == 0 {
                    continue;
                }

                let (risk, confidence) = if age >= 30 {
                    (RiskLevel::Low, 0.95)
                } else if age >= 7 {
                    (RiskLevel::Low, 0.85)
                } else {
                    (RiskLevel::Low, 0.60)
                };

                let reason = format!(
                    "Temp file unused for {} day{}",
                    age,
                    if age == 1 { "" } else { "s" }
                );

                findings.push(
                    Finding::new(
                        self.id(),
                        "temp_file_age",
                        Category::TempFiles,
                        risk,
                        TargetKind::File,
                        path.to_string_lossy(),
                    )
                    .with_size(size)
                    .with_age(age)
                    .with_confidence(confidence)
                    .with_reason(reason)
                    .with_action("delete"),
                );
            }
        }

        // Highest confidence first
        findings.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn scanner_id_and_name() {
        assert_eq!(TempDirScanner.id(), "temp_dirs");
        assert!(!TempDirScanner.name().is_empty());
        assert!(!TempDirScanner.description().is_empty());
    }

    #[test]
    fn does_not_require_elevation() {
        assert!(!TempDirScanner.requires_elevation());
    }

    #[test]
    fn skips_brand_new_files() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("fresh.tmp"), b"data").unwrap();

        let mut ctx = ScanContext::new();
        ctx.target_paths = vec![dir.path().to_path_buf()];

        let findings = TempDirScanner.scan(&ctx).unwrap();
        assert!(findings.is_empty(), "fresh files (age=0) must not be reported");
    }

    #[test]
    fn returns_ok_on_nonexistent_path() {
        let mut ctx = ScanContext::new();
        ctx.target_paths = vec![PathBuf::from(r"Z:\NonExistentTempDir")];
        let result = TempDirScanner.scan(&ctx);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn empty_dir_yields_no_findings() {
        let dir = TempDir::new().unwrap();
        let mut ctx = ScanContext::new();
        ctx.target_paths = vec![dir.path().to_path_buf()];
        let findings = TempDirScanner.scan(&ctx).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn temp_paths_not_empty() {
        // Must resolve at least one path from env vars on any Windows machine
        let paths = TempDirScanner::temp_paths();
        assert!(!paths.is_empty());
    }

    #[test]
    fn temp_paths_no_duplicates() {
        let paths = TempDirScanner::temp_paths();
        let mut seen = std::collections::HashSet::new();
        for p in &paths {
            assert!(seen.insert(p.clone()), "duplicate temp path: {:?}", p);
        }
    }

    #[test]
    fn findings_sorted_by_confidence_descending() {
        // Create a dir with fresh files only — just checks the sort logic compiles
        // (age=0 files are skipped, so we verify we get an empty sorted vec)
        let dir = TempDir::new().unwrap();
        let mut ctx = ScanContext::new();
        ctx.target_paths = vec![dir.path().to_path_buf()];
        let findings = TempDirScanner.scan(&ctx).unwrap();
        // Verify sorted: each confidence >= next
        for w in findings.windows(2) {
            assert!(w[0].confidence >= w[1].confidence);
        }
    }

    #[test]
    fn blacklisted_paths_are_skipped() {
        // Even if we somehow point the scanner at System32, nothing should come out
        let mut ctx = ScanContext::new();
        ctx.target_paths = vec![PathBuf::from(r"C:\Windows\System32")];
        let findings = TempDirScanner.scan(&ctx).unwrap();
        assert!(findings.is_empty(), "blacklisted paths must produce zero findings");
    }
}

use std::path::PathBuf;

use crate::{
    blacklist,
    models::{Category, Finding, RiskLevel, ScanContext, TargetKind},
    Result, Scanner,
};

/// Scans for Windows Update leftover files and old patches.
/// Phase 1: read-only analysis only. Phase 2 will integrate with DISM.
pub struct WindowsUpdateScanner;

impl WindowsUpdateScanner {
    fn update_paths() -> Vec<PathBuf> {
        vec![
            PathBuf::from(r"C:\Windows\SoftwareDistribution\Download"),
            PathBuf::from(r"C:\ProgramData\Microsoft\Windows\WER\ReportArchive"),
        ]
    }
}

impl Scanner for WindowsUpdateScanner {
    fn id(&self) -> &'static str {
        "windows_update"
    }

    fn name(&self) -> &'static str {
        "Windows Update Leftovers"
    }

    fn description(&self) -> &'static str {
        "Finds old Windows Update patches and temporary files (read-only in Phase 1)."
    }

    fn scan(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let roots = if ctx.target_paths.is_empty() {
            Self::update_paths()
        } else {
            ctx.target_paths.clone()
        };

        let mut findings = Vec::new();

        for root in &roots {
            if !root.exists() {
                continue;
            }

            let Ok(entries) = std::fs::read_dir(root) else {
                continue;
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if !blacklist::is_path_safe(&path) {
                    continue;
                }

                let Ok(meta) = entry.metadata() else {
                    continue;
                };

                let size = if meta.is_file() {
                    meta.len()
                } else if meta.is_dir() {
                    std::fs::read_dir(&path)
                        .map(|e| {
                            e.flatten()
                                .filter_map(|f| f.metadata().ok().map(|m| m.len()))
                                .sum()
                        })
                        .unwrap_or(0)
                } else {
                    continue;
                };

                if size == 0 {
                    continue;
                }

                findings.push(
                    Finding::new(
                        self.id(),
                        "windows_update_leftover",
                        Category::WindowsUpdate,
                        RiskLevel::Low,
                        if meta.is_file() {
                            TargetKind::File
                        } else {
                            TargetKind::Directory
                        },
                        path.to_string_lossy(),
                    )
                    .with_size(size)
                    .with_confidence(0.85)
                    .with_reason("Old Windows Update file or patch")
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

    #[test]
    fn scanner_id() {
        assert_eq!(WindowsUpdateScanner.id(), "windows_update");
    }

    #[test]
    fn no_elevation_needed() {
        assert!(!WindowsUpdateScanner.requires_elevation());
    }
}

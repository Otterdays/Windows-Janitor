use std::path::PathBuf;

use crate::{
    blacklist,
    models::{Category, Finding, RiskLevel, ScanContext, TargetKind},
    Result, Scanner,
};

/// Scans for application crash dumps (`.dmp` files and error logs).
pub struct CrashDumpScanner;

impl CrashDumpScanner {
    fn crash_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            paths.push(PathBuf::from(local).join("CrashDumps"));
        }

        if let Ok(appdata) = std::env::var("APPDATA") {
            paths.push(PathBuf::from(appdata).join("Microsoft\\Windows\\WER\\ReportArchive"));
        }

        paths
    }
}

impl Scanner for CrashDumpScanner {
    fn id(&self) -> &'static str {
        "crash_dumps"
    }

    fn name(&self) -> &'static str {
        "Crash Dumps"
    }

    fn description(&self) -> &'static str {
        "Finds application crash dump files (.dmp, .log) that can be safely removed."
    }

    fn scan(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let roots = if ctx.target_paths.is_empty() {
            Self::crash_paths()
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

                if !meta.is_file() {
                    continue;
                }

                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                if !filename.ends_with(".dmp") && !filename.ends_with(".log") {
                    continue;
                }

                let size = meta.len();
                if size == 0 {
                    continue;
                }

                findings.push(
                    Finding::new(
                        self.id(),
                        "crash_dump_file",
                        Category::CrashDumps,
                        RiskLevel::Low,
                        TargetKind::File,
                        path.to_string_lossy(),
                    )
                    .with_size(size)
                    .with_confidence(0.95)
                    .with_reason("Application crash dump or error log")
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
        assert_eq!(CrashDumpScanner.id(), "crash_dumps");
    }

    #[test]
    fn does_not_require_elevation() {
        assert!(!CrashDumpScanner.requires_elevation());
    }
}

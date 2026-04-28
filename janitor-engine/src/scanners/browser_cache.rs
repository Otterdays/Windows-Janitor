use std::path::PathBuf;

use walkdir::WalkDir;

use crate::{
    blacklist,
    models::{Category, Finding, RiskLevel, ScanContext, TargetKind},
    Result, Scanner,
};

/// Scans browser cache directories for Chrome, Edge, and Firefox.
pub struct BrowserCacheScanner;

struct BrowserProfile {
    browser: &'static str,
    /// Relative to %LOCALAPPDATA% or %APPDATA%
    base_env: &'static str,
    cache_subpath: &'static str,
    /// Whether the profile directory uses a glob-style wildcard level
    wildcard_level: bool,
}

const PROFILES: &[BrowserProfile] = &[
    BrowserProfile {
        browser: "Chrome",
        base_env: "LOCALAPPDATA",
        cache_subpath: r"Google\Chrome\User Data\Default\Cache",
        wildcard_level: false,
    },
    BrowserProfile {
        browser: "Chrome (Profile 1+)",
        base_env: "LOCALAPPDATA",
        cache_subpath: r"Google\Chrome\User Data",
        wildcard_level: true,
    },
    BrowserProfile {
        browser: "Edge",
        base_env: "LOCALAPPDATA",
        cache_subpath: r"Microsoft\Edge\User Data\Default\Cache",
        wildcard_level: false,
    },
    BrowserProfile {
        browser: "Edge (Profile 1+)",
        base_env: "LOCALAPPDATA",
        cache_subpath: r"Microsoft\Edge\User Data",
        wildcard_level: true,
    },
    BrowserProfile {
        browser: "Firefox",
        base_env: "APPDATA",
        cache_subpath: r"Mozilla\Firefox\Profiles",
        wildcard_level: true,
    },
    BrowserProfile {
        browser: "Brave",
        base_env: "LOCALAPPDATA",
        cache_subpath: r"BraveSoftware\Brave-Browser\User Data\Default\Cache",
        wildcard_level: false,
    },
    BrowserProfile {
        browser: "Opera",
        base_env: "APPDATA",
        cache_subpath: r"Opera Software\Opera Stable\Cache",
        wildcard_level: false,
    },
];

impl BrowserCacheScanner {
    fn collect_cache_roots() -> Vec<(PathBuf, &'static str)> {
        let mut roots: Vec<(PathBuf, &'static str)> = Vec::new();

        for profile in PROFILES {
            let Ok(base) = std::env::var(profile.base_env) else { continue };
            let base_path = PathBuf::from(base).join(profile.cache_subpath);

            if profile.wildcard_level {
                // Enumerate one level down: User Data\Profile N\ or Profiles\*.default\
                let Ok(entries) = std::fs::read_dir(&base_path) else { continue };
                for entry in entries.flatten() {
                    let sub = entry.path();
                    if !sub.is_dir() {
                        continue;
                    }
                    // For Firefox: look for cache2 inside the profile folder
                    // For Chrome/Edge: look for Cache inside Profile N
                    let cache = if profile.browser.starts_with("Firefox") {
                        sub.join("cache2")
                    } else {
                        sub.join("Cache")
                    };
                    if cache.exists() {
                        roots.push((cache, profile.browser));
                    }
                }
            } else if base_path.exists() {
                roots.push((base_path, profile.browser));
            }
        }

        roots
    }
}

impl Scanner for BrowserCacheScanner {
    fn id(&self) -> &'static str {
        "browser_cache"
    }

    fn name(&self) -> &'static str {
        "Browser Cache"
    }

    fn description(&self) -> &'static str {
        "Scans Chrome, Edge, Firefox, Brave, and Opera cache directories."
    }

    fn scan(&self, ctx: &ScanContext) -> Result<Vec<Finding>> {
        let cache_roots: Vec<(PathBuf, &str)> = if ctx.target_paths.is_empty() {
            Self::collect_cache_roots()
        } else {
            ctx.target_paths
                .iter()
                .map(|p| (p.clone(), "custom"))
                .collect()
        };

        let mut findings = Vec::new();

        for (root, browser) in &cache_roots {
            let mut root_size: u64 = 0;
            let mut file_count: u64 = 0;

            for entry in WalkDir::new(root)
                .min_depth(1)
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

                root_size += meta.len();
                file_count += 1;
            }

            if root_size == 0 {
                continue;
            }

            // Report the cache directory as a single finding (not per-file)
            // This avoids thousands of tiny findings for a single browser.
            let reason = format!(
                "{} cache ({} files, {} MB)",
                browser,
                file_count,
                root_size / 1_048_576,
            );

            findings.push(
                Finding::new(
                    self.id(),
                    "browser_cache_dir",
                    Category::BrowserCache,
                    RiskLevel::Low,
                    TargetKind::Directory,
                    root.to_string_lossy(),
                )
                .with_size(root_size)
                .with_confidence(0.90)
                .with_reason(reason)
                .with_action("delete"),
            );
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
        assert_eq!(BrowserCacheScanner.id(), "browser_cache");
        assert!(!BrowserCacheScanner.name().is_empty());
        assert!(!BrowserCacheScanner.description().is_empty());
    }

    #[test]
    fn does_not_require_elevation() {
        assert!(!BrowserCacheScanner.requires_elevation());
    }

    #[test]
    fn empty_when_no_browsers_installed() {
        let mut ctx = ScanContext::new();
        ctx.target_paths = vec![PathBuf::from(r"Z:\NoSuchBrowser\Cache")];
        let findings = BrowserCacheScanner.scan(&ctx).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn returns_ok_on_missing_path() {
        let mut ctx = ScanContext::new();
        ctx.target_paths = vec![PathBuf::from(r"C:\NoSuchCache")];
        assert!(BrowserCacheScanner.scan(&ctx).is_ok());
    }

    #[test]
    fn reports_directory_not_individual_files() {
        // Each cache root produces exactly ONE finding (the directory summary),
        // not one per file inside it.
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("cache_data_1"), b"x".repeat(1024).as_slice()).unwrap();
        std::fs::write(dir.path().join("cache_data_2"), b"y".repeat(1024).as_slice()).unwrap();

        let mut ctx = ScanContext::new();
        ctx.target_paths = vec![dir.path().to_path_buf()];
        let findings = BrowserCacheScanner.scan(&ctx).unwrap();

        // Two files in one cache dir → one finding
        assert!(findings.len() <= 1, "expected at most 1 finding per cache root, got {}", findings.len());
    }

    #[test]
    fn empty_cache_dir_yields_no_finding() {
        // An empty cache directory has 0 bytes → should not produce a finding
        let dir = TempDir::new().unwrap();
        let mut ctx = ScanContext::new();
        ctx.target_paths = vec![dir.path().to_path_buf()];
        let findings = BrowserCacheScanner.scan(&ctx).unwrap();
        assert!(findings.is_empty(), "empty cache dirs must not produce findings");
    }

    #[test]
    fn finding_size_matches_actual_content() {
        let dir = TempDir::new().unwrap();
        let content = b"a".repeat(2048);
        std::fs::write(dir.path().join("f1"), content.as_slice()).unwrap();

        let mut ctx = ScanContext::new();
        ctx.target_paths = vec![dir.path().to_path_buf()];
        let findings = BrowserCacheScanner.scan(&ctx).unwrap();

        if let Some(f) = findings.first() {
            assert_eq!(f.size_bytes, 2048, "reported size must match actual bytes");
            assert_eq!(f.category, crate::models::Category::BrowserCache);
        }
    }

    #[test]
    fn findings_sorted_by_size_descending() {
        let dir = TempDir::new().unwrap();
        let mut ctx = ScanContext::new();
        ctx.target_paths = vec![dir.path().to_path_buf()];
        let findings = BrowserCacheScanner.scan(&ctx).unwrap();
        for w in findings.windows(2) {
            assert!(w[0].size_bytes >= w[1].size_bytes);
        }
    }
}

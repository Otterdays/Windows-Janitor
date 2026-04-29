pub mod browser_cache;
pub mod crash_dumps;
pub mod recycle_bin;
pub mod temp_dirs;
pub mod windows_update;

pub use browser_cache::BrowserCacheScanner;
pub use crash_dumps::CrashDumpScanner;
pub use recycle_bin::RecycleBinScanner;
pub use temp_dirs::TempDirScanner;
pub use windows_update::WindowsUpdateScanner;

use crate::Scanner;

/// Returns all built-in scanners in default priority order.
pub fn all_scanners() -> Vec<Box<dyn Scanner>> {
    vec![
        Box::new(TempDirScanner),
        Box::new(RecycleBinScanner),
        Box::new(BrowserCacheScanner),
        Box::new(CrashDumpScanner),
        Box::new(WindowsUpdateScanner),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_scanners_returns_expected_count() {
        assert_eq!(all_scanners().len(), 5);
    }

    #[test]
    fn all_scanner_ids_are_unique() {
        let scanners = all_scanners();
        let mut ids: Vec<&str> = scanners.iter().map(|s| s.id()).collect();
        let original_len = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "scanner IDs must be unique");
    }

    #[test]
    fn all_scanners_have_nonempty_metadata() {
        for s in all_scanners() {
            assert!(!s.id().is_empty(), "scanner id must not be empty");
            assert!(!s.name().is_empty(), "scanner name must not be empty");
            assert!(!s.description().is_empty(), "scanner description must not be empty");
        }
    }

    #[test]
    fn known_scanner_ids_present() {
        let scanners = all_scanners();
        let ids: Vec<&str> = scanners.iter().map(|s| s.id()).collect();
        assert!(ids.contains(&"temp_dirs"));
        assert!(ids.contains(&"recycle_bin"));
        assert!(ids.contains(&"browser_cache"));
        assert!(ids.contains(&"crash_dumps"));
        assert!(ids.contains(&"windows_update"));
    }

    #[test]
    fn no_scanner_requires_elevation_by_default() {
        // All Phase 1 scanners should run without admin
        for s in all_scanners() {
            assert!(
                !s.requires_elevation(),
                "scanner '{}' unexpectedly requires elevation",
                s.id()
            );
        }
    }
}

pub mod browser_cache;
pub mod recycle_bin;
pub mod temp_dirs;

pub use browser_cache::BrowserCacheScanner;
pub use recycle_bin::RecycleBinScanner;
pub use temp_dirs::TempDirScanner;

use crate::Scanner;

/// Returns all built-in scanners in default priority order.
pub fn all_scanners() -> Vec<Box<dyn Scanner>> {
    vec![
        Box::new(TempDirScanner),
        Box::new(RecycleBinScanner),
        Box::new(BrowserCacheScanner),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_scanners_returns_expected_count() {
        assert_eq!(all_scanners().len(), 3);
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

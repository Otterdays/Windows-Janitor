//! Janitor engine — core scanning, classification, and quarantine logic.
//!
//! This is the heart of Janitor. Pure Rust, no UI dependencies.
//! Implements Scanner trait, rules engine, finding classification, and quarantine manifest.

pub mod blacklist;
pub mod error;
pub mod models;
pub mod scanner;
pub mod scanners;

pub use error::{Error, Result};
pub use models::{Category, Finding, RiskLevel, ScanContext, ScanResult, TargetKind};
pub use scanner::Scanner;
pub use scanners::all_scanners;

/// Engine version. Incremented when scanner or rule behavior changes.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_compiles() {
        assert!(!ENGINE_VERSION.is_empty());
    }
}

//! Hard blacklist: safety boundary that cannot be overridden.
//!
//! These paths are never safe to touch, regardless of rules or scanner output.
//! This is defense-in-depth: scanners should not generate findings for these paths,
//! but the blacklist is checked before any action is taken.

use std::path::{Component, Path, PathBuf};

/// Check if a path is on the hard blacklist.
/// Returns true if the path MUST NOT be touched.
pub fn is_path_safe(path: &Path) -> bool {
    !is_blacklisted(path)
}

/// Check if a path is blacklisted (must not be touched).
/// Uses both exact-match and prefix-based rules.
pub fn is_blacklisted(path: &Path) -> bool {
    let normalized = normalize_path(path);
    let lower = normalized.to_string_lossy().to_lowercase();

    // System-critical paths (never touch)
    if lower.starts_with("c:\\windows\\winsxs") {
        return true; // Component store — DISM only
    }
    if lower.starts_with("c:\\windows\\system32") {
        return true; // System binaries
    }
    if lower.starts_with("c:\\windows\\syswow64") {
        return true; // 32-bit system binaries on x64
    }
    if lower.starts_with("c:\\windows\\servicing") {
        return true; // Windows servicing engine
    }
    if lower.starts_with("c:\\windows\\assembly") {
        return true; // Legacy assembly cache
    }

    // Kernel resources (never safe to touch)
    if lower == "c:\\pagefile.sys"
        || lower == "c:\\hiberfil.sys"
        || lower == "c:\\swapfile.sys"
    {
        return true; // Kernel-controlled memory files
    }

    // Boot loader
    if lower.starts_with("c:\\boot") {
        return true;
    }

    // BitLocker
    if lower.starts_with("c:\\\\?\\volume{") {
        return true; // BitLocker metadata
    }

    // Boot configuration data
    if lower.starts_with("\\\\?\\globalroot\\device\\harddiskvolume") {
        return true;
    }

    false
}

/// Normalize a path for comparison: resolve .. and ., remove trailing slashes, lowercase.
fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => {
                result.push(prefix.as_os_str());
            }
            Component::RootDir => {
                result.push(component);
            }
            Component::Normal(c) => {
                result.push(c);
            }
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {
                // Skip
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Blacklisted paths ---

    #[test]
    fn winsxs_is_blacklisted() {
        assert!(!is_path_safe(Path::new("C:\\Windows\\WinSxS\\amd64_foo")));
        assert!(!is_path_safe(Path::new("c:\\windows\\winsxs\\x86_bar")));
    }

    #[test]
    fn system32_is_blacklisted() {
        assert!(!is_path_safe(Path::new("C:\\Windows\\System32\\kernel32.dll")));
        assert!(!is_path_safe(Path::new("C:\\Windows\\SysWOW64\\msvcrt.dll")));
    }

    #[test]
    fn syswow64_is_blacklisted() {
        assert!(!is_path_safe(Path::new("C:\\Windows\\SysWOW64")));
        assert!(!is_path_safe(Path::new("C:\\WINDOWS\\SYSWOW64\\foo.dll")));
    }

    #[test]
    fn servicing_is_blacklisted() {
        assert!(!is_path_safe(Path::new("C:\\Windows\\Servicing\\Packages")));
    }

    #[test]
    fn assembly_cache_is_blacklisted() {
        assert!(!is_path_safe(Path::new("C:\\Windows\\Assembly\\NativeImages")));
    }

    #[test]
    fn kernel_files_are_blacklisted() {
        assert!(!is_path_safe(Path::new("C:\\pagefile.sys")));
        assert!(!is_path_safe(Path::new("C:\\hiberfil.sys")));
        assert!(!is_path_safe(Path::new("C:\\swapfile.sys")));
    }

    #[test]
    fn boot_dir_is_blacklisted() {
        assert!(!is_path_safe(Path::new("C:\\Boot\\BCD")));
        assert!(!is_path_safe(Path::new("C:\\boot\\memtest.exe")));
    }

    // --- Safe paths ---

    #[test]
    fn safe_paths_pass() {
        assert!(is_path_safe(Path::new("C:\\Users\\User\\AppData\\Local\\Temp")));
        assert!(is_path_safe(Path::new("C:\\ProgramData\\temp")));
        assert!(is_path_safe(Path::new("D:\\Downloads\\old_installer.exe")));
    }

    #[test]
    fn windows_temp_is_safe() {
        // C:\Windows\Temp is NOT in the blacklist — scanners are allowed to touch it
        assert!(is_path_safe(Path::new("C:\\Windows\\Temp\\old.tmp")));
    }

    #[test]
    fn other_drives_safe_by_default() {
        assert!(is_path_safe(Path::new("D:\\Downloads\\file.zip")));
        assert!(is_path_safe(Path::new("E:\\Backup\\archive.7z")));
    }

    #[test]
    fn recycle_bin_is_safe() {
        // $Recycle.Bin is safe to scan — we enumerate and delete from it
        assert!(is_path_safe(Path::new("C:\\$Recycle.Bin\\S-1-5-21\\$Rfoo")));
    }

    // --- Case insensitivity ---

    #[test]
    fn case_insensitive() {
        assert!(!is_path_safe(Path::new("C:\\WINDOWS\\WINSXS\\foo")));
        assert!(!is_path_safe(Path::new("c:\\windows\\winsxs\\foo")));
        assert!(!is_path_safe(Path::new("C:\\Windows\\WinSxS\\foo")));
    }

    // --- Path traversal resistance ---

    #[test]
    fn normalize_resolves_dots() {
        let p = Path::new("C:\\Windows\\..\\System32\\kernel32.dll");
        let normalized = normalize_path(p);
        let s = normalized.to_string_lossy();
        assert!(!s.contains(".."));
    }

    #[test]
    fn path_traversal_into_system32_is_blocked() {
        // Attacker tries: C:\Users\Temp\..\..\..\Windows\System32\evil.dll
        let p = Path::new("C:\\Users\\Temp\\..\\..\\..\\Windows\\System32\\evil.dll");
        // After normalization this resolves to C:\Windows\System32\evil.dll → blacklisted
        assert!(!is_path_safe(p));
    }

    #[test]
    fn path_traversal_into_winsxs_is_blocked() {
        let p = Path::new("C:\\Users\\..\\Windows\\WinSxS\\foo");
        assert!(!is_path_safe(p));
    }

    // --- is_blacklisted mirrors is_path_safe ---

    #[test]
    fn is_blacklisted_is_inverse_of_is_path_safe() {
        let paths = [
            "C:\\Windows\\System32\\ntdll.dll",
            "C:\\Users\\User\\AppData\\Local\\Temp\\foo.tmp",
            "C:\\pagefile.sys",
            "D:\\Downloads\\setup.exe",
        ];
        for p in &paths {
            let path = Path::new(p);
            assert_eq!(is_blacklisted(path), !is_path_safe(path), "mismatch on {}", p);
        }
    }
}

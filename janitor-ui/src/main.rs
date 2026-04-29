// Prevent extra console window on Windows release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Instant;

use janitor_engine::{all_scanners, models::ScanResult, ScanContext};
use rayon::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct ScannerInfo {
    id: String,
    name: String,
    description: String,
    requires_elevation: bool,
}

#[tauri::command]
fn list_scanners() -> Vec<ScannerInfo> {
    all_scanners()
        .iter()
        .map(|s| ScannerInfo {
            id: s.id().to_string(),
            name: s.name().to_string(),
            description: s.description().to_string(),
            requires_elevation: s.requires_elevation(),
        })
        .collect()
}

#[tauri::command]
fn run_scan(scanner_id: Option<String>, dev_caches: bool) -> ScanResult {
    let mut ctx = ScanContext::new();
    ctx.include_dev_caches = dev_caches;

    let scanners = all_scanners();
    let filtered: Vec<_> = scanners
        .iter()
        .filter(|s| match &scanner_id {
            Some(id) => s.id() == id,
            None => true,
        })
        .collect();

    let start = Instant::now();
    let results: Vec<_> = filtered
        .par_iter()
        .map(|s| (s.id(), s.scan(&ctx)))
        .collect();
    let elapsed_ms = start.elapsed().as_millis() as u64;

    let mut report = ScanResult::new(&ctx.scan_id);
    report.duration_ms = elapsed_ms;

    for (id, result) in results {
        match result {
            Ok(mut findings) => report.findings.append(&mut findings),
            Err(e) => report.errors.push(format!("[{}] {}", id, e)),
        }
    }

    report
}

#[tauri::command]
fn engine_version() -> String {
    janitor_engine::ENGINE_VERSION.to_string()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_scanners,
            run_scan,
            engine_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

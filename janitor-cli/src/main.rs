use std::time::Instant;

use clap::{Parser, Subcommand};
use janitor_engine::{
    all_scanners, models::ScanResult, RiskLevel, ScanContext,
};
use rayon::prelude::*;

#[derive(Parser)]
#[command(
    name = "janitor",
    about = "Safe, transparent Windows PC cleaner",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scan for junk files and report findings (read-only)
    Scan {
        /// Output results as JSON
        #[arg(long)]
        json: bool,

        /// Include developer caches (npm, cargo, pip, etc.)
        #[arg(long)]
        dev_caches: bool,

        /// Run only a specific scanner by ID (e.g. temp_dirs, recycle_bin, browser_cache)
        #[arg(long)]
        scanner: Option<String>,
    },

    /// List available scanners
    List,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::List => {
            let scanners = all_scanners();
            println!("{:<20} {:<8} {}", "ID", "ELEV", "DESCRIPTION");
            println!("{}", "-".repeat(72));
            for s in &scanners {
                println!(
                    "{:<20} {:<8} {}",
                    s.id(),
                    if s.requires_elevation() { "yes" } else { "no" },
                    s.description()
                );
            }
        }

        Command::Scan {
            json,
            dev_caches,
            scanner: filter,
        } => {
            let mut ctx = ScanContext::new();
            ctx.include_dev_caches = dev_caches;

            let scanners = all_scanners();
            let scanners: Vec<_> = scanners
                .iter()
                .filter(|s| {
                    if let Some(ref id) = filter {
                        s.id() == id
                    } else {
                        true
                    }
                })
                .collect();

            if scanners.is_empty() {
                eprintln!("Error: no matching scanner found");
                std::process::exit(1);
            }

            if !json {
                eprintln!("Janitor — scanning {} scanner(s)...", scanners.len());
            }

            let start = Instant::now();

            let results: Vec<_> = scanners
                .par_iter()
                .map(|s| {
                    let findings = s.scan(&ctx);
                    (s.id(), findings)
                })
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

            if json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                print_human(&report);
            }
        }
    }
}

fn print_human(report: &ScanResult) {
    let total_mb = report.total_reclaimable_bytes() as f64 / 1_048_576.0;
    let high = report.count_by_risk(RiskLevel::High);
    let medium = report.count_by_risk(RiskLevel::Medium);
    let low = report.count_by_risk(RiskLevel::Low);

    println!();
    println!("  Janitor Scan Report");
    println!("  Scan ID : {}", report.scan_id);
    println!("  Duration: {}ms", report.duration_ms);
    println!(
        "  Found   : {} findings  ({:.1} MB reclaimable)",
        report.findings.len(),
        total_mb
    );
    println!(
        "  Risk    : {} high  {} medium  {} low",
        high, medium, low
    );

    if !report.errors.is_empty() {
        println!();
        println!("  Errors ({}):", report.errors.len());
        for e in &report.errors {
            println!("    ! {}", e);
        }
    }

    if report.findings.is_empty() {
        println!();
        println!("  No findings. System looks clean.");
        return;
    }

    println!();
    println!(
        "  {:<16} {:<8} {:<8} {:<8} {}",
        "SCANNER", "RISK", "SIZE MB", "AGE(d)", "PATH"
    );
    println!("  {}", "-".repeat(88));

    // Show top 50 by size to keep output readable
    let mut sorted = report.findings.clone();
    sorted.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    for f in sorted.iter().take(50) {
        let size_mb = f.size_bytes as f64 / 1_048_576.0;
        let path = if f.target_ref.len() > 50 {
            format!("...{}", &f.target_ref[f.target_ref.len() - 47..])
        } else {
            f.target_ref.clone()
        };
        println!(
            "  {:<16} {:<8} {:<8.1} {:<8} {}",
            f.scanner_id, f.risk, size_mb, f.age_days, path
        );
    }

    if report.findings.len() > 50 {
        println!(
            "  ... and {} more (use --json for full output)",
            report.findings.len() - 50
        );
    }

    println!();
    println!("  NOTE: This is a read-only report. No files were modified.");
    println!();
}

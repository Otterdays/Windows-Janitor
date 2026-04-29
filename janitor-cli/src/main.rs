use std::path::PathBuf;
use std::time::Instant;

use clap::{Parser, Subcommand};
use janitor_engine::{
    all_scanners, models::ScanResult, RiskLevel, ScanContext,
};
use rayon::prelude::*;

#[derive(Parser)]
#[command(
    name = "janitor",
    about = "Safe, transparent Windows PC cleaner (read-only in Phase 1)",
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

        /// Export results to an HTML file
        #[arg(long)]
        html: Option<PathBuf>,

        /// Export results to a JSON file
        #[arg(long)]
        output: Option<PathBuf>,

        /// Include developer caches (npm, cargo, pip, etc.)
        #[arg(long)]
        dev_caches: bool,

        /// Run only a specific scanner by ID
        #[arg(long)]
        scanner: Option<String>,

        /// Filter by minimum file size (MB)
        #[arg(long)]
        min_size_mb: Option<f64>,

        /// Filter by category (case-insensitive)
        #[arg(long)]
        category: Option<String>,

        /// Filter by risk level (low, medium, high)
        #[arg(long)]
        risk: Option<String>,
    },

    /// List available scanners
    List,

    /// Show program info and safety notes
    About,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::List => {
            let scanners = all_scanners();
            println!("\nAvailable scanners:\n");
            println!("{:<20} {:<8} DESCRIPTION", "ID", "ELEV");
            println!("{}", "-".repeat(72));
            for s in &scanners {
                println!(
                    "{:<20} {:<8} {}",
                    s.id(),
                    if s.requires_elevation() { "yes" } else { "no" },
                    s.description()
                );
            }
            println!();
        }

        Command::About => {
            println!();
            println!("  Janitor v0.1 — Safe Windows PC Cleaner");
            println!();
            println!("  SAFETY:");
            println!("    • This version is READ-ONLY — no files are modified.");
            println!("    • All paths are checked against a hard safety blacklist.");
            println!("    • System files (System32, WinSxS, etc.) are never touched.");
            println!("    • Findings show confidence scores (0.0-1.0) indicating certainty.");
            println!();
            println!("  USAGE:");
            println!("    janitor scan                 # Human-readable report");
            println!("    janitor scan --json          # JSON output (pipe to file)");
            println!("    janitor scan --html out.html # HTML report");
            println!("    janitor scan --min-size-mb 10 # Only findings >= 10 MB");
            println!("    janitor scan --category TempFiles");
            println!("    janitor scan --risk high     # Only high-risk findings");
            println!("    janitor list                 # Show all scanners");
            println!();
            println!("  SOURCE: https://github.com/yourusername/janitor");
            println!();
        }

        Command::Scan {
            json,
            html,
            output,
            dev_caches,
            scanner: filter,
            min_size_mb,
            category: cat_filter,
            risk: risk_filter,
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

            if !json && html.is_none() && output.is_none() {
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

            // Apply filters
            report.findings.retain(|f| {
                if let Some(min_mb) = min_size_mb {
                    if (f.size_bytes as f64 / 1_048_576.0) < min_mb {
                        return false;
                    }
                }
                if let Some(ref cat) = cat_filter {
                    if !f.category.to_string().to_lowercase().contains(&cat.to_lowercase()) {
                        return false;
                    }
                }
                if let Some(ref risk) = risk_filter {
                    let matches = match risk.to_lowercase().as_str() {
                        "low" => f.risk == RiskLevel::Low,
                        "medium" => f.risk == RiskLevel::Medium,
                        "high" => f.risk == RiskLevel::High,
                        _ => true,
                    };
                    if !matches {
                        return false;
                    }
                }
                true
            });

            // Output
            if let Some(ref path) = output {
                let json_str = serde_json::to_string_pretty(&report).unwrap();
                std::fs::write(path, json_str).expect("Failed to write JSON file");
                eprintln!("Report written to: {}", path.display());
            }

            if let Some(ref path) = html {
                let html_str = generate_html(&report);
                std::fs::write(path, html_str).expect("Failed to write HTML file");
                eprintln!("Report written to: {}", path.display());
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else if output.is_none() && html.is_none() {
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
        "  {:<16} {:<8} {:<8} {:<8} PATH",
        "SCANNER", "RISK", "SIZE MB", "AGE(d)"
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

fn generate_html(report: &ScanResult) -> String {
    let total_mb = report.total_reclaimable_bytes() as f64 / 1_048_576.0;
    let high = report.count_by_risk(RiskLevel::High);
    let medium = report.count_by_risk(RiskLevel::Medium);
    let low = report.count_by_risk(RiskLevel::Low);

    let mut sorted = report.findings.clone();
    sorted.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    let mut rows = String::new();
    for f in &sorted {
        let size_mb = f.size_bytes as f64 / 1_048_576.0;
        let risk_color = match f.risk {
            RiskLevel::High => "#ff4444",
            RiskLevel::Medium => "#ff9900",
            RiskLevel::Low => "#44ff44",
        };
        rows.push_str(&format!(
            "<tr><td style='background-color:{}'>{}</td><td>{}</td><td>{:.1} MB</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            risk_color, f.risk, f.scanner_id, size_mb, f.age_days, f.category, f.target_ref
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Janitor Scan Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        h1 {{ color: #333; }}
        .summary {{ background: #f0f0f0; padding: 15px; border-radius: 5px; margin-bottom: 20px; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 10px; text-align: left; }}
        th {{ background-color: #4CAF50; color: white; }}
        tr:hover {{ background-color: #f5f5f5; }}
        .note {{ color: #666; font-style: italic; margin-top: 20px; }}
    </style>
</head>
<body>
    <h1>Janitor Scan Report</h1>
    <div class="summary">
        <p><strong>Scan ID:</strong> {}</p>
        <p><strong>Duration:</strong> {}ms</p>
        <p><strong>Total Findings:</strong> {}</p>
        <p><strong>Reclaimable Space:</strong> {:.1} MB</p>
        <p><strong>Risk Distribution:</strong> {} High | {} Medium | {} Low</p>
    </div>
    <table>
        <tr>
            <th>Risk</th>
            <th>Scanner</th>
            <th>Size</th>
            <th>Age (days)</th>
            <th>Category</th>
            <th>Path</th>
        </tr>
        {}
    </table>
    <p class="note">This is a read-only report. No files were modified.</p>
</body>
</html>"#,
        report.scan_id,
        report.duration_ms,
        report.findings.len(),
        total_mb,
        high,
        medium,
        low,
        rows
    )
}

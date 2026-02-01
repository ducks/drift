use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

use drift::{run_audit, Issue};

#[derive(Parser)]
#[command(name = "drift")]
#[command(
    about = "Repo drift auditor - checks for stale configs, version mismatches, dead code markers, and CI/local drift"
)]
#[command(version)]
struct Cli {
    /// Output results as JSON
    #[arg(short, long)]
    json: bool,

    /// Directory to audit (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Err(e) = std::env::set_current_dir(&cli.path) {
        eprintln!("Error: Cannot access directory {:?}: {}", cli.path, e);
        return ExitCode::FAILURE;
    }

    let issues = run_audit();

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&issues).unwrap());
    } else {
        print_human_readable(&issues);
    }

    if issues.iter().any(|i| i.severity == "error") {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn print_human_readable(issues: &[Issue]) {
    if issues.is_empty() {
        println!("✓ No drift detected");
        return;
    }

    println!("Drift Audit Results");
    println!("===================\n");

    for issue in issues {
        let icon = match issue.severity.as_str() {
            "error" => "✗",
            "warning" => "⚠",
            _ => "○",
        };

        print!("{} [{}] {}", icon, issue.category, issue.message);
        if let Some(ref path) = issue.path {
            print!(" ({})", path.display());
        }
        if let Some(line) = issue.line {
            print!(":{}", line);
        }
        println!();
    }

    let errors = issues.iter().filter(|i| i.severity == "error").count();
    let warnings = issues.iter().filter(|i| i.severity == "warning").count();
    println!("\nSummary: {} errors, {} warnings", errors, warnings);
}

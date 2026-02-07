use crate::ui::{self, colors, icons};
use anyhow::Result;
use crossterm::style::Stylize;

#[allow(clippy::unnecessary_wraps)]
pub fn run(all: bool) -> Result<()> {
    ui::print_header("  Security Audit  ");

    println!(
        "\n{} {}",
        icons::SHIELD.with(colors::CYAN),
        "Checking available scanners...".with(colors::WHITE)
    );

    let mut available = Vec::new();
    let mut unavailable = Vec::new();

    // Check for semgrep
    if which_command("semgrep") {
        available.push(("semgrep", "Static analysis for multiple languages"));
    } else {
        unavailable.push(("semgrep", "pip install semgrep"));
    }

    // Check for trivy
    if which_command("trivy") {
        available.push(("trivy", "Vulnerability scanner for containers/filesystems"));
    } else {
        unavailable.push(("trivy", "brew install trivy"));
    }

    // Check for cargo-audit
    if which_command("cargo-audit") {
        available.push(("cargo-audit", "Rust dependency vulnerability scanner"));
    } else {
        unavailable.push(("cargo-audit", "cargo install cargo-audit"));
    }

    // Show available scanners
    if !available.is_empty() {
        ui::print_section(icons::CHECK, "Available Scanners");
        for (name, desc) in &available {
            println!(
                "  {} {} {}",
                icons::DOT.with(colors::GREEN),
                name.bold().with(colors::GREEN),
                format!("- {desc}").with(colors::DIM)
            );
        }
    }

    // Show unavailable scanners
    if !unavailable.is_empty() {
        ui::print_section(icons::CROSS, "Not Installed");
        for (name, install) in &unavailable {
            println!(
                "  {} {} {}",
                icons::DOT.with(colors::DIM),
                name.with(colors::DIM),
                format!("({install})").with(colors::DIM)
            );
        }
    }

    // Run scans if --all
    if all && !available.is_empty() {
        println!();
        ui::print_section(icons::GAUGE, "Running Scans");

        for (name, _) in &available {
            println!(
                "\n  {} Running {}...",
                icons::ARROW.with(colors::CYAN),
                name.bold().with(colors::WHITE)
            );

            let result = match *name {
                "semgrep" => run_semgrep(),
                "trivy" => run_trivy(),
                "cargo-audit" => run_cargo_audit(),
                _ => Ok(()),
            };

            match result {
                Ok(()) => {
                    println!(
                        "  {} {} complete",
                        icons::CHECK.with(colors::GREEN),
                        name.with(colors::GREEN)
                    );
                }
                Err(e) => {
                    println!(
                        "  {} {} failed: {}",
                        icons::CROSS.with(colors::RED),
                        name.with(colors::RED),
                        e.to_string().with(colors::DIM)
                    );
                }
            }
        }
    } else if !all {
        println!(
            "\n{} Use {} to run all available scanners",
            icons::ARROW.with(colors::DIM),
            "--all".with(colors::YELLOW)
        );
    }

    Ok(())
}

fn which_command(cmd: &str) -> bool {
    #[cfg(windows)]
    let result = std::process::Command::new("where.exe").arg(cmd).output();
    #[cfg(not(windows))]
    let result = std::process::Command::new("which").arg(cmd).output();
    result.is_ok_and(|o| o.status.success())
}

fn run_semgrep() -> Result<()> {
    let output = std::process::Command::new("semgrep")
        .args(["scan", "--config", "auto", ".", "--quiet"])
        .output()?;

    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    Ok(())
}

fn run_trivy() -> Result<()> {
    let output = std::process::Command::new("trivy")
        .args(["fs", ".", "--quiet"])
        .output()?;

    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    Ok(())
}

fn run_cargo_audit() -> Result<()> {
    // Only run if Cargo.toml exists
    if !std::path::Path::new("Cargo.toml").exists() {
        return Ok(());
    }

    let output = std::process::Command::new("cargo")
        .args(["audit", "--quiet"])
        .output()?;

    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    Ok(())
}

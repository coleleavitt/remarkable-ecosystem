//! reMarkable Firmware Patcher
//!
//! Binary patcher for modifying xochitl to enable hidden features,
//! remove telemetry, and customize behavior.

mod binary;
mod cli;
mod error;
mod patch;
mod patterns;
mod signature;

use clap::Parser;
use colored::Colorize;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("remarkable_patcher=info".parse()?))
        .init();

    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Analyze { binary } => {
            cmd_analyze(&binary)?;
        }
        cli::Commands::Patch {
            binary,
            output,
            patches,
            all,
            no_backup,
        } => {
            cmd_patch(&binary, output.as_deref(), &patches, all, no_backup)?;
        }
        cli::Commands::List => {
            cmd_list_patches();
        }
        cli::Commands::Verify { binary } => {
            cmd_verify(&binary)?;
        }
        cli::Commands::Restore { backup } => {
            cmd_restore(&backup)?;
        }
        cli::Commands::Extract { binary, output } => {
            cmd_extract(&binary, &output)?;
        }
    }

    Ok(())
}

fn cmd_analyze(binary_path: &std::path::Path) -> Result<(), error::PatcherError> {
    use binary::XochitlBinary;

    println!("{}", "═══════════════════════════════════════════════════════════════".cyan());
    println!("{}", " reMarkable xochitl Binary Analysis".cyan().bold());
    println!("{}", "═══════════════════════════════════════════════════════════════".cyan());

    let xochitl = XochitlBinary::open(binary_path)?;
    let info = xochitl.info();

    println!("\n{}", "Binary Information:".yellow().bold());
    println!("  Path:        {}", info.path.display());
    println!("  Size:        {} bytes ({:.2} MB)", info.size, info.size as f64 / 1_048_576.0);
    println!("  SHA256:      {}", &info.sha256[..16]);
    println!("  Architecture: {}", info.architecture);
    println!("  Build ID:    {}", info.build_id.as_deref().unwrap_or("unknown"));

    println!("\n{}", "Version Detection:".yellow().bold());
    if let Some(version) = &info.version {
        println!("  Detected:    {} {}", "✓".green(), version.green());
    } else {
        println!("  Detected:    {} {}", "?".yellow(), "Unknown version".yellow());
    }

    println!("\n{}", "Patch Status:".yellow().bold());
    let patches = patch::all_patches();
    for p in &patches {
        let status = xochitl.check_patch(&**p)?;
        let status_str = match status {
            patch::PatchStatus::Applied => "Applied".green(),
            patch::PatchStatus::NotApplied => "Not Applied".yellow(),
            patch::PatchStatus::Unknown => "Unknown".red(),
        };
        println!("  {:30} {}", p.name(), status_str);
    }

    println!("\n{}", "Pattern Scan Results:".yellow().bold());
    let scanner = patterns::PatternScanner::new(&xochitl);
    for target in patterns::SCAN_TARGETS {
        match scanner.find_pattern(target) {
            Some(offset) => println!("  {:30} {} @ 0x{:08x}", target.name, "Found".green(), offset),
            None => println!("  {:30} {}", target.name, "Not Found".red()),
        }
    }

    Ok(())
}

fn cmd_patch(
    binary_path: &std::path::Path,
    output: Option<&std::path::Path>,
    patches_to_apply: &[String],
    all: bool,
    no_backup: bool,
) -> Result<(), error::PatcherError> {
    use binary::XochitlBinary;

    println!("{}", "═══════════════════════════════════════════════════════════════".cyan());
    println!("{}", " reMarkable xochitl Patcher".cyan().bold());
    println!("{}", "═══════════════════════════════════════════════════════════════".cyan());

    let mut xochitl = XochitlBinary::open(binary_path)?;
    let info = xochitl.info();

    println!("\n{} {}", "Target:".yellow().bold(), info.path.display());
    if let Some(v) = &info.version {
        println!("{} {}", "Version:".yellow().bold(), v);
    }

    // Create backup unless disabled
    if !no_backup {
        let backup_path = xochitl.create_backup()?;
        println!("{} {}", "Backup:".green().bold(), backup_path.display());
    }

    // Determine which patches to apply
    let all_patches = patch::all_patches();
    let patches: Vec<_> = if all {
        all_patches.iter().collect()
    } else {
        all_patches
            .iter()
            .filter(|p| patches_to_apply.iter().any(|name| p.matches_name(name)))
            .collect()
    };

    if patches.is_empty() {
        println!("{}", "\nNo patches selected. Use --all or specify patch names.".yellow());
        return Ok(());
    }

    println!("\n{}", "Applying patches:".yellow().bold());

    let mut applied = 0;
    for p in patches {
        print!("  {:30} ", p.name());
        match xochitl.apply_patch(&**p)? {
            patch::PatchResult::Applied { offset } => {
                println!("{} @ 0x{:08x}", "Applied".green(), offset);
                applied += 1;
            }
            patch::PatchResult::AlreadyApplied => {
                println!("{}", "Already Applied".yellow());
            }
            patch::PatchResult::PatternNotFound => {
                println!("{}", "Pattern Not Found".red());
            }
        }
    }

    // Write output
    let output_path = output.unwrap_or(binary_path);
    xochitl.write_to(output_path)?;

    println!("\n{} Applied {} patches", "✓".green().bold(), applied);
    println!("{} {}", "Output:".green().bold(), output_path.display());

    // Verify checksums
    let new_checksum = xochitl.checksum();
    println!("{} {}", "New SHA256:".green().bold(), &new_checksum[..16]);

    Ok(())
}

fn cmd_list_patches() {
    println!("{}", "═══════════════════════════════════════════════════════════════".cyan());
    println!("{}", " Available Patches".cyan().bold());
    println!("{}", "═══════════════════════════════════════════════════════════════".cyan());

    for p in patch::all_patches() {
        println!("\n{}", p.name().yellow().bold());
        println!("  {}", p.description());
        println!("  Category: {}", format!("{:?}", p.category()).cyan());
        if let Some(risk) = p.risk_level() {
            let risk_str = match risk {
                patch::RiskLevel::Low => "Low".green(),
                patch::RiskLevel::Medium => "Medium".yellow(),
                patch::RiskLevel::High => "High".red(),
            };
            println!("  Risk: {}", risk_str);
        }
    }
}

fn cmd_verify(binary_path: &std::path::Path) -> Result<(), error::PatcherError> {
    use binary::XochitlBinary;

    println!("{}", "═══════════════════════════════════════════════════════════════".cyan());
    println!("{}", " Binary Verification".cyan().bold());
    println!("{}", "═══════════════════════════════════════════════════════════════".cyan());

    let xochitl = XochitlBinary::open(binary_path)?;

    // Check if it's a valid xochitl binary
    if xochitl.is_valid_xochitl() {
        println!("\n{} Valid xochitl binary", "✓".green().bold());
    } else {
        println!("\n{} Not a valid xochitl binary", "✗".red().bold());
        return Ok(());
    }

    // Check known signatures
    let sig_db = signature::SignatureDatabase::load_builtin();
    if let Some(entry) = sig_db.lookup(&xochitl.checksum()) {
        println!("{} Known firmware version: {}", "✓".green().bold(), entry.version);
        println!("  Device: {}", entry.device);
        println!("  Status: {}", entry.status);
    } else {
        println!("{} Unknown firmware version (SHA256 not in database)", "?".yellow().bold());
    }

    Ok(())
}

fn cmd_restore(backup_path: &std::path::Path) -> Result<(), error::PatcherError> {
    use binary::XochitlBinary;

    println!("{}", "═══════════════════════════════════════════════════════════════".cyan());
    println!("{}", " Restore from Backup".cyan().bold());
    println!("{}", "═══════════════════════════════════════════════════════════════".cyan());

    XochitlBinary::restore_backup(backup_path)?;

    println!("\n{} Restored from {}", "✓".green().bold(), backup_path.display());

    Ok(())
}

fn cmd_extract(binary_path: &std::path::Path, output_dir: &std::path::Path) -> Result<(), error::PatcherError> {
    use binary::XochitlBinary;

    println!("{}", "═══════════════════════════════════════════════════════════════".cyan());
    println!("{}", " Extract Resources".cyan().bold());
    println!("{}", "═══════════════════════════════════════════════════════════════".cyan());

    let xochitl = XochitlBinary::open(binary_path)?;
    let extracted = xochitl.extract_resources(output_dir)?;

    println!("\n{} Extracted {} resources to {}", "✓".green().bold(), extracted, output_dir.display());

    Ok(())
}

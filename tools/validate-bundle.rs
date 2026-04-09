#!/usr/bin/env -S cargo +stable -Zscript
---
[dependencies]
flate2 = "1.0"
tar = "0.4"
---

//! Bundle validation helper for DSView CLI releases.
//!
//! Validates that an unpacked bundle contains:
//! - dsview-cli executable
//! - runtime/ directory with target-specific runtime library
//! - resources/ directory with required DSLogic Plus files
//! - Executable smoke tests pass (--help, devices list --help)
//!
//! Usage:
//!   cargo +stable -Zscript tools/validate-bundle.rs \
//!     --archive dsview-cli-v0.1.0-x86_64-unknown-linux-gnu.tar.gz \
//!     --target x86_64-unknown-linux-gnu

use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use flate2::read::GzDecoder;
use tar::Archive;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = parse_args(&args).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        eprintln!("\nUsage: cargo +stable -Zscript tools/validate-bundle.rs \\");
        eprintln!("  --archive <archive.tar.gz> \\");
        eprintln!("  --target <triple>");
        std::process::exit(1);
    });

    validate_bundle(&config).unwrap_or_else(|e| {
        eprintln!("Validation failed: {}", e);
        std::process::exit(1);
    });

    println!("✓ Bundle validation passed");
}

struct ValidateConfig {
    archive_path: PathBuf,
    target: String,
}

fn parse_args(args: &[String]) -> Result<ValidateConfig, String> {
    let mut archive_path = None;
    let mut target = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--archive" => {
                i += 1;
                archive_path = Some(PathBuf::from(&args[i]));
            }
            "--target" => {
                i += 1;
                target = Some(args[i].clone());
            }
            _ => return Err(format!("Unknown argument: {}", args[i])),
        }
        i += 1;
    }

    Ok(ValidateConfig {
        archive_path: archive_path.ok_or("--archive is required")?,
        target: target.ok_or("--target is required")?,
    })
}

fn validate_bundle(config: &ValidateConfig) -> io::Result<()> {
    if !config.archive_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Archive not found: {}", config.archive_path.display()),
        ));
    }

    // Extract to temp directory
    let temp_dir = std::env::temp_dir().join(format!(
        "dsview-validate-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir)?;

    let tar_gz = File::open(&config.archive_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(&temp_dir)?;

    // Find the bundle root directory
    let entries: Vec<_> = fs::read_dir(&temp_dir)?
        .filter_map(|e| e.ok())
        .collect();

    if entries.len() != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Expected single root directory, found {} entries", entries.len()),
        ));
    }

    let bundle_root = entries[0].path();
    if !bundle_root.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Archive root is not a directory",
        ));
    }

    // Validate executable
    let exe_name = if config.target.contains("windows") {
        "dsview-cli.exe"
    } else {
        "dsview-cli"
    };
    let exe_path = bundle_root.join(exe_name);
    if !exe_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Executable not found: {}", exe_path.display()),
        ));
    }

    // Validate runtime directory
    let runtime_dir = bundle_root.join("runtime");
    if !runtime_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "runtime/ directory not found",
        ));
    }

    // Validate runtime library
    let runtime_name = runtime_library_name(&config.target);
    let runtime_path = runtime_dir.join(runtime_name);
    if !runtime_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Runtime library not found: {}", runtime_path.display()),
        ));
    }

    // Validate resources directory
    let resources_dir = bundle_root.join("resources");
    if !resources_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "resources/ directory not found",
        ));
    }

    // Validate required DSLogic Plus resources
    let required_resources = [
        "DSLogicPlus.fw",
        "DSLogicPlus.bin",
        "DSLogicPlus-pgl12.bin",
    ];

    for resource in &required_resources {
        let resource_path = resources_dir.join(resource);
        if !resource_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Required resource not found: {}", resource),
            ));
        }
    }

    // Smoke test: --help
    let help_status = Command::new(&exe_path)
        .arg("--help")
        .status();

    match help_status {
        Ok(status) if status.success() => {
            println!("✓ dsview-cli --help passed");
        }
        Ok(status) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("dsview-cli --help failed with exit code: {:?}", status.code()),
            ));
        }
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to run dsview-cli --help: {}", e),
            ));
        }
    }

    // Smoke test: devices list --help
    let devices_help_status = Command::new(&exe_path)
        .args(&["devices", "list", "--help"])
        .status();

    match devices_help_status {
        Ok(status) if status.success() => {
            println!("✓ dsview-cli devices list --help passed");
        }
        Ok(status) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("dsview-cli devices list --help failed with exit code: {:?}", status.code()),
            ));
        }
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to run dsview-cli devices list --help: {}", e),
            ));
        }
    }

    // Clean up temp directory
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

fn runtime_library_name(target: &str) -> &str {
    if target.contains("windows") {
        "dsview_runtime.dll"
    } else if target.contains("darwin") || target.contains("macos") {
        "libdsview_runtime.dylib"
    } else {
        "libdsview_runtime.so"
    }
}

#!/usr/bin/env -S cargo +stable -Zscript
---
[dependencies]
flate2 = "1.0"
tar = "0.4"
---

//! Bundle packaging helper for DSView CLI releases.
//!
//! Creates a versioned archive containing:
//! - dsview-cli executable
//! - runtime/ directory with the target-specific runtime library
//! - resources/ directory with DSLogic Plus firmware and bitstreams
//!
//! Usage:
//!   cargo +stable -Zscript tools/package-bundle.rs \
//!     --exe target/release/dsview-cli \
//!     --runtime target/release/build/dsview-sys-*/out/source-runtime-build/libdsview_runtime.so \
//!     --resources DSView/DSView/res \
//!     --output dsview-cli-v0.1.0-x86_64-unknown-linux-gnu.tar.gz \
//!     --version v0.1.0 \
//!     --target x86_64-unknown-linux-gnu

use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::Builder;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = parse_args(&args).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        eprintln!("\nUsage: cargo +stable -Zscript tools/package-bundle.rs \\");
        eprintln!("  --exe <path> \\");
        eprintln!("  --runtime <path> \\");
        eprintln!("  --resources <dir> \\");
        eprintln!("  --output <archive.tar.gz> \\");
        eprintln!("  --version <version> \\");
        eprintln!("  --target <triple>");
        std::process::exit(1);
    });

    package_bundle(&config).unwrap_or_else(|e| {
        eprintln!("Packaging failed: {}", e);
        std::process::exit(1);
    });

    println!("Bundle created: {}", config.output.display());
}

struct PackageConfig {
    exe_path: PathBuf,
    runtime_path: PathBuf,
    resources_dir: PathBuf,
    output: PathBuf,
    version: String,
    target: String,
}

fn parse_args(args: &[String]) -> Result<PackageConfig, String> {
    let mut exe_path = None;
    let mut runtime_path = None;
    let mut resources_dir = None;
    let mut output = None;
    let mut version = None;
    let mut target = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--exe" => {
                i += 1;
                exe_path = Some(PathBuf::from(&args[i]));
            }
            "--runtime" => {
                i += 1;
                runtime_path = Some(PathBuf::from(&args[i]));
            }
            "--resources" => {
                i += 1;
                resources_dir = Some(PathBuf::from(&args[i]));
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(&args[i]));
            }
            "--version" => {
                i += 1;
                version = Some(args[i].clone());
            }
            "--target" => {
                i += 1;
                target = Some(args[i].clone());
            }
            _ => return Err(format!("Unknown argument: {}", args[i])),
        }
        i += 1;
    }

    Ok(PackageConfig {
        exe_path: exe_path.ok_or("--exe is required")?,
        runtime_path: runtime_path.ok_or("--runtime is required")?,
        resources_dir: resources_dir.ok_or("--resources is required")?,
        output: output.ok_or("--output is required")?,
        version: version.ok_or("--version is required")?,
        target: target.ok_or("--target is required")?,
    })
}

fn package_bundle(config: &PackageConfig) -> io::Result<()> {
    if !config.exe_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Executable not found: {}", config.exe_path.display()),
        ));
    }
    if !config.runtime_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Runtime library not found: {}", config.runtime_path.display()),
        ));
    }
    if !config.resources_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Resources directory not found: {}", config.resources_dir.display()),
        ));
    }

    let archive_root = format!("dsview-cli-{}-{}", config.version, config.target);

    let tar_gz = File::create(&config.output)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut ar = Builder::new(enc);

    // Add executable
    let exe_name = if config.target.contains("windows") {
        "dsview-cli.exe"
    } else {
        "dsview-cli"
    };
    let mut exe_file = File::open(&config.exe_path)?;
    ar.append_file(
        format!("{}/{}", archive_root, exe_name),
        &mut exe_file,
    )?;

    // Add runtime library
    let runtime_name = config.runtime_path.file_name().unwrap();
    let mut runtime_file = File::open(&config.runtime_path)?;
    ar.append_file(
        format!("{}/runtime/{}", archive_root, runtime_name.to_string_lossy()),
        &mut runtime_file,
    )?;

    // Add DSLogic Plus resources only
    let required_resources = [
        "DSLogicPlus.fw",
        "DSLogic.fw",  // fallback
        "DSLogicPlus.bin",
        "DSLogicPlus-pgl12.bin",
    ];

    for resource in &required_resources {
        let resource_path = config.resources_dir.join(resource);
        if resource_path.exists() {
            let mut resource_file = File::open(&resource_path)?;
            ar.append_file(
                format!("{}/resources/{}", archive_root, resource),
                &mut resource_file,
            )?;
        }
    }

    ar.finish()?;
    Ok(())
}

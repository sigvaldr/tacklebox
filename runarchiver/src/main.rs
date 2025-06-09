// ------------------------------------------------------------------------------
// Author        : Sigvaldr Nótthrafn
// Project       : runarchiver
// File          : main.rs
// Creation Date : 08JUN2025
// ------------------------------------------------------------------------------

use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process;

use tar::Archive;
use zstd::stream::read::Decoder;

const VERSION: &str = "1.0.0";
fn main() {
    println!("RustUnArchiver v{} by Sigvaldr", VERSION);
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <archive.tar.zst> [-to <output_folder>]", args[0]);
        process::exit(1);
    }

    let archive_path = &args[1];
    let output_folder = parse_output_folder(&args);

    if let Err(e) = extract_archive(archive_path, &output_folder) {
        eprintln!("Extraction failed: {}", e);
        process::exit(1);
    }

    println!("Archive extracted to '{}'", output_folder.display());
}

fn parse_output_folder(args: &[String]) -> PathBuf {
    if let Some(pos) = args.iter().position(|arg| arg == "-to") {
        if let Some(folder) = args.get(pos + 1) {
            return PathBuf::from(folder);
        } else {
            eprintln!("Error: -to flag requires a folder name");
            process::exit(1);
        }
    }

    // Default: use archive name without `.tar.zst`
    let archive_name = Path::new(&args[1]);
    let stem = archive_name
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();

    // Handle double extensions like `.tar.zst`
    let folder_name = if stem.ends_with(".tar") {
        stem.strip_suffix(".tar").unwrap_or(&stem)
    } else {
        &stem
    };

    PathBuf::from(folder_name)
}

fn extract_archive(
    archive_path: &str,
    output_folder: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let oname = output_folder.to_string_lossy().to_string();
    println!("Extracting {} into {}", archive_path, oname);
    let archive_file = File::open(archive_path)?;
    let decoder = Decoder::new(archive_file)?;
    let mut archive = Archive::new(decoder);

    std::fs::create_dir_all(output_folder)?;
    archive.unpack(output_folder)?;
    Ok(())
}

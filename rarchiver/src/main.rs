// ------------------------------------------------------------------------------
// Author        : Sigvaldr Nótthrafn
// Project       : rarchiver
// File          : main.rs
// Creation Date : 08JUN2025
// ------------------------------------------------------------------------------

use chrono::Local;
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process;
use tar::Builder;
use zstd::stream::Encoder;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!(
            "Usage: [-stamp] {} <input_folder> [output_archive.tar.zst]",
            args[0]
        );
        process::exit(1);
    }

    let input_path = &args[1];

    if !Path::new(input_path).is_dir() {
        eprintln!("Error: '{}' is not a valid directory.", input_path);
        process::exit(1);
    }

    let mut add_stamp = false;
    let mut output_path: Option<PathBuf> = None;

    // Process optional output name and -stamp flag
    for arg in &args[2..] {
        if arg == "-stamp" {
            add_stamp = true;
        } else if output_path.is_none() {
            output_path = Some(PathBuf::from(arg));
        }
    }

    // Auto-name the archive based on folder if no output file given
    let mut final_output = if let Some(path) = output_path {
        path
    } else {
        auto_generate_filename(input_path)
    };

    if add_stamp {
        final_output = stamp_filename(&final_output);
    }

    match compress_folder(input_path, &final_output) {
        Ok(_) => println!("Archive created: {}", final_output.display()),
        Err(e) => {
            eprintln!("Compression failed: {}", e);
            process::exit(1);
        }
    }
}

fn stamp_filename(original: &Path) -> PathBuf {
    let date = Local::now().format("%d%b%Y").to_string().to_uppercase(); // "01JAN2025"

    let stem = original.file_stem().unwrap_or_default().to_string_lossy();
    let ext = original.extension().unwrap_or_default().to_string_lossy();

    let parent = original.parent().unwrap_or_else(|| Path::new(""));
    let new_name = if ext.is_empty() {
        format!("{date}-{stem}")
    } else {
        format!("{date}-{stem}.{}", ext)
    };

    parent.join(new_name)
}

fn auto_generate_filename(input_folder: &str) -> PathBuf {
    let path = Path::new(input_folder);
    let folder_name = path.file_name().unwrap_or_default().to_string_lossy();
    let archive_name = format!("{}.tar.zst", folder_name);
    PathBuf::from(archive_name)
}

fn compress_folder(
    input_folder: &str,
    output_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let tar_gz = File::create(output_file)?;
    let encoder = Encoder::new(tar_gz, 22)?; // Max compression level
    let mut tar_builder = Builder::new(encoder);

    tar_builder.append_dir_all(".", input_folder)?;
    let encoder = tar_builder.into_inner()?; // Finish writing tar
    encoder.finish()?; // Finish compression

    Ok(())
}

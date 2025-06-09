// ------------------------------------------------------------------------------
// Author        : Sigvaldr Nótthrafn
// Project       : rarchiver
// File          : main.rs
// Creation Date : 08JUN2025
// ------------------------------------------------------------------------------

use chrono::Local;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process;
use std::{env, fs};
use tar::Builder;
use zstd::stream::Encoder;

const VERSION: &str = "1.0.0";

fn main() {
    println!("RustArchiver v{} by Sigvaldr", VERSION);
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!(
            "Usage: [-stamp] {} <input_folder> [output_archive.box]",
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

    for arg in &args[2..] {
        if arg == "-stamp" {
            add_stamp = true;
        } else if output_path.is_none() {
            output_path = Some(PathBuf::from(arg));
        }
    }

    let mut final_output = if let Some(path) = output_path {
        set_box_extension(path)
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

fn set_box_extension(mut path: PathBuf) -> PathBuf {
    path.set_extension("box");
    path
}

fn stamp_filename(original: &Path) -> PathBuf {
    let date = Local::now().format("%d%b%Y").to_string().to_uppercase();
    let stem = original.file_stem().unwrap_or_default().to_string_lossy();
    let parent = original.parent().unwrap_or_else(|| Path::new(""));
    let new_name = format!("{date}-{stem}.box");

    parent.join(new_name)
}

fn auto_generate_filename(input_folder: &str) -> PathBuf {
    let path = Path::new(input_folder);
    let folder_name = path.file_name().unwrap_or_default().to_string_lossy();
    PathBuf::from(format!("{}.box", folder_name))
}

fn compress_folder(
    input_folder: &str,
    output_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Compressing {} into {}...",
        input_folder,
        output_file.display()
    );

    // Create a temp .tar.zst file to store actual archive
    let temp_path = output_file.with_extension("tar.zst");
    let tar_zst = File::create(&temp_path)?;
    let encoder = Encoder::new(tar_zst, 22)?;
    let mut tar_builder = Builder::new(encoder);

    tar_builder.append_dir_all(".", input_folder)?;
    let encoder = tar_builder.into_inner()?;
    encoder.finish()?;

    // Rename to .box
    fs::rename(temp_path, output_file)?;

    println!("Done.");
    Ok(())
}

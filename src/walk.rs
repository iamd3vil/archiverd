use std::fs;
use tar::Builder;

use crate::args::Args;
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use flate2::write::ZlibEncoder;
use glob::Pattern;
use rayon::prelude::*;

pub fn run_archive_loop(args: &Args) -> Result<()> {
    let dir = Utf8PathBuf::try_from(&args.directory).context("converting to Utf8PathBuf")?;
    let exclude_glob = args
        .exclude
        .as_ref()
        .map(|e| Pattern::new(e).context("valid glob pattern"))
        .transpose()?;
    let latest_file = latest_file(&dir, &exclude_glob)?;

    let paths: Vec<_> = fs::read_dir(&args.directory)?.collect::<Result<_, _>>()?;

    // Loop over the files in the directory in parallel.
    paths.par_iter().try_for_each(|entry| -> Result<()> {
        let path = Utf8PathBuf::try_from(entry.path()).context("converting to Utf8PathBuf")?;

        let meta = fs::metadata(&path)?;
        if meta.is_dir() {
            return Ok(());
        }

        if path.as_str().ends_with(".tar.gz") {
            return Ok(());
        }

        // Check if the entry is the latest file.
        if path == latest_file {
            println!("Skipping latest file: {:?}", path);
            return Ok(());
        }

        if let Some(exclude) = &exclude_glob {
            // Check if the file name matches the exclude glob pattern.
            if exclude.matches(path.as_str()) {
                println!("Skipping excluded file: {:?}", path);
                return Ok(());
            }
        }

        // Create a tar file with the file.
        let tar_path = Utf8PathBuf::try_from(format!("{}.tar", path))?;
        println!("Creating tar file: {:?}", tar_path);

        let file = fs::File::create(&tar_path)?;
        let mut builder = Builder::new(file);

        // Add the file to the tar.
        builder.append_path_with_name(&path, &path.file_name().unwrap())?;

        // Finish writing the tar file.
        builder.finish()?;

        // Create a gzip file with the tar file using the flate2 crate.
        let gz_path = tar_path.with_extension("tar.gz");
        println!("Creating gz file: {:?}", gz_path);

        let tar_file = fs::File::open(&tar_path)?;
        let gz_file = fs::File::create(&gz_path)?;
        let mut encoder = ZlibEncoder::new(gz_file, flate2::Compression::default());

        std::io::copy(&mut std::io::BufReader::new(tar_file), &mut encoder)?;

        // Delete the tar file.
        fs::remove_file(&tar_path)?;

        // Delete the file.
        fs::remove_file(&path)?;

        Ok(())
    })?;

    Ok(())
}

// Get file path of the latest file by creation in the directory.
fn latest_file(dir: &Utf8PathBuf, exclude: &Option<glob::Pattern>) -> Result<Utf8PathBuf> {
    let mut latest: Option<Utf8PathBuf> = None;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = Utf8PathBuf::try_from(entry.path()).context("converting to Utf8PathBuf")?;

        let meta = fs::metadata(&path)?;
        if meta.is_dir() {
            continue;
        }

        if path.as_str().ends_with(".tar.gz") {
            continue;
        }

        if let Some(exclude) = exclude {
            // Check if the file name matches the exclude glob pattern.
            if exclude.matches(path.as_str()) {
                continue;
            }
        }

        if let Some(ref lat) = latest {
            if path.metadata()?.created()? > lat.metadata()?.created()? {
                latest = Some(path);
            }
        } else {
            latest = Some(path);
        }
    }

    latest.ok_or_else(|| anyhow::anyhow!("No files in directory"))
}

use human_size::Size;
use std::{
    fs::{self, DirEntry},
    io::Write,
    path::Path,
    time::SystemTime,
};
use tar::Builder;

use crate::args::Args;
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use flate2::bufread::GzEncoder;
use glob::Pattern;
use rayon::prelude::*;

pub fn run_archive_loop(args: &Args) -> Result<()> {
    let dir = Utf8PathBuf::from(&args.directory);
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

        // Check if min size is given and the file is of minimum size.
        if let Some(min_size_str) = &args.keep_min_size {
            // Parse the give size into bytes.
            let min_size: Size = min_size_str.parse().context("couldn't parse min size")?;
            println!("min size: {min_size}");

            keep_file_min_size(&path, min_size.value() as u64)?;
            return Ok(());
        }

        // Create a tar file with the file.
        let tar_path = Utf8PathBuf::from(format!("{}.tar", path));
        println!("Creating tar file: {:?}", tar_path);

        let file = fs::File::create(&tar_path)?;
        let mut builder = Builder::new(file);

        // Add the file to the tar.
        builder.append_path_with_name(&path, path.file_name().unwrap())?;

        // Finish writing the tar file.
        builder.finish()?;

        // Create a gzip file with the tar file using the flate2 crate.
        let gz_path = tar_path.with_extension("tar.gz");
        println!("Creating gz file: {:?}", gz_path);

        let tar_file = fs::File::open(&tar_path)?;
        let gz_file = fs::File::create(&gz_path)?;
        let mut gz_writer = std::io::BufWriter::new(gz_file);
        let gz_reader = std::io::BufReader::new(tar_file);
        let mut encoder = GzEncoder::new(gz_reader, flate2::Compression::default());

        std::io::copy(&mut encoder, &mut gz_writer)?;

        gz_writer.flush()?;

        // Delete the tar file.
        fs::remove_file(&tar_path)?;

        // Delete the file.
        fs::remove_file(&path)?;

        Ok(())
    })?;

    // Keep only the latest n files.
    if let Some(max_files) = args.max_files {
        keep_latest_n_files(&dir, max_files)?;
    }

    Ok(())
}

fn keep_latest_n_files<P: AsRef<Path>>(dir: P, n: usize) -> Result<()> {
    let mut entries: Vec<DirEntry> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let p = Utf8PathBuf::try_from(entry.path()).unwrap();
            p.to_string().ends_with(".tar.gz")
        })
        .collect();

    // Sort files by modified time, newest first
    entries.sort_by(|a, b| {
        let a_time = a
            .metadata()
            .and_then(|m| m.created())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let b_time = b
            .metadata()
            .and_then(|m| m.created())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        b_time.cmp(&a_time)
    });

    // Keep only the latest n files
    for entry in entries.into_iter().skip(n) {
        fs::remove_file(entry.path())?;
    }

    Ok(())
}

fn keep_file_min_size(file: &Utf8PathBuf, min_size: u64) -> Result<()> {
    let meta = fs::metadata(file)?;
    if meta.len() < min_size {
        println!("Deleting file: {:?}", file);
        fs::remove_file(file)?;
    }

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

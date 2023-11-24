mod args;
mod walk;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use args::Args;
use camino::Utf8PathBuf;
use clap::Parser;
use notify::event::CreateKind;
use notify::Watcher;

use crate::walk::run_archive_loop;

fn main() -> Result<()> {
    let args = Arc::new(Args::parse());

    println!("Using directory: {}", args.directory);
    println!("Maximum files to process: {:?}", args.max_files);

    if let Err(e) = walk::run_archive_loop(&args) {
        eprintln!("Error: {}", e);
    }

    if args.disable_watch {
        return Ok(());
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let args = args.clone();
    let dir_path = Utf8PathBuf::from(&args.directory);
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => {
            if let Err(err) = handle_notify_event(event, &args) {
                println!("watch error: {:?}", err);
            }
        }
        Err(e) => println!("watch error: {:?}", e),
    })?;
    watcher.watch(dir_path.as_std_path(), notify::RecursiveMode::NonRecursive)?;

    println!("Waiting for Ctrl-C...");
    while running.load(Ordering::SeqCst) {}
    println!("Got it! Exiting...");

    Ok(())
}

fn handle_notify_event(event: notify::Event, args: &Args) -> Result<()> {
    match event.kind {
        notify::EventKind::Create(CreateKind::File) => {
            println!("Created: {:?}", event.paths);

            run_archive_loop(args)?;
        }
        _ => {}
    }

    Ok(())
}

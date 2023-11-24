mod args;
mod walk;

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};

use anyhow::Result;
use args::Args;
use camino::Utf8PathBuf;
use clap::Parser;
use notify::event::CreateKind;
use notify::{EventKind, Watcher};

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

    let (tx, rx): (Sender<u32>, Receiver<u32>) = mpsc::channel();

    ctrlc::set_handler(move || {
        let _ = tx.send(0);
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
    let _ = rx.recv().unwrap();
    println!("Got it! Exiting...");

    Ok(())
}

fn handle_notify_event(event: notify::Event, args: &Args) -> Result<()> {
    if let EventKind::Create(CreateKind::File) = event.kind {
        println!("Created: {:?}", event.paths);

        run_archive_loop(args)?;
    }

    Ok(())
}

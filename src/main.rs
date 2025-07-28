use anyhow::{Context, Result};
use clap::Parser;
use log::{info, warn};
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;

#[derive(Parser)]
struct Cli {
    pattern: String,
    path: std::path::PathBuf,
    #[command(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,
}

fn main() -> Result<()> {
    // interrupt handler
    let should_continue = Arc::new(AtomicBool::new(true));
    let ctrlc_should_continue = should_continue.clone();
    ctrlc::set_handler(move || {
        ctrlc_should_continue.swap(false, Ordering::SeqCst);
    })?;

    let args = Cli::parse();

    env_logger::Builder::new()
        .filter_level(args.verbosity.into())
        .init();
    info!("starting up");

    let f = std::fs::File::open(&args.path)
        .with_context(|| format!("could not read file `{}`", args.path.display()))?;
    let reader = std::io::BufReader::new(f);

    let stdout = io::stdout();
    let mut stdout_handle = io::BufWriter::new(stdout.lock());

    for l in reader.lines() {
        if !should_continue.load(Ordering::SeqCst) {
            break;
        }

        match l {
            Ok(line_content) => {
                prip::find_matches(&line_content, &args.pattern, &mut stdout_handle)?;
            }
            Err(error) => {
                eprintln!("Error: {}", error);
            }
        }
    }
    stdout_handle.flush()?;

    let pb = indicatif::ProgressBar::new(100);
    for i in 0..100 {
        pb.println(format!("[+] finished #{}", i));
        pb.inc(1);
        if !should_continue.load(Ordering::SeqCst) {
            break;
        }
        sleep(std::time::Duration::from_millis(5));
    }
    pb.finish_with_message("done");
    Ok(())
}

#[test]
fn check_if_the_world_exists() {
    assert_eq!("world", "world");
}

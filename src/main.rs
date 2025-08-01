use anyhow::{Context, Result};
use clap::Parser;
use log::trace;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,
    output: String,

    #[arg(
        short = 'd',
        long = "delete",
        help = "Delete files from device after copy"
    )]
    delete: bool,
}

fn main() -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    let args = Cli::parse();

    env_logger::Builder::new()
        .filter_level(args.verbosity.into())
        .init();

    trace!("environment initialized");

    let ctx = prip::Context::new()?;

    let cameras = ctx
        .list_cameras()
        .with_context(|| format!("could not list attached camera devices"))?;

    let selected_camera_descriptor = inquire::Select::new("Choose a camera:", cameras).prompt()?;

    let camera = ctx
        .get_camera(selected_camera_descriptor)
        .with_context(|| format!("could not fetch the selected camera"))?;

    trace!("selected camera summary:\n{}", camera.get_summary()?);
    trace!("selected port: {}", camera.get_port()?);

    let output_path = Path::new(&args.output);

    let mut delete_confirmed = false;
    if args.delete {
        delete_confirmed = inquire::Confirm::new(
            "Delete flag is enabled, this will delete files on device after copying.",
        )
        .with_default(false)
        .prompt()?;
    }

    // TODO: make it handle ctrlc properly
    camera.move_all_files("/", output_path, delete_confirmed)?;

    Ok(())
}

use anyhow::{Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use log::trace;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Parser)]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,

    #[arg(short = 'o', long = "output", help = "Output directory path")]
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
        .with_context(|| format!("Could not list attached camera devices"))?;

    if cameras.is_empty() {
        anyhow::bail!("No cameras found");
    }

    let selected_camera_descriptor = inquire::Select::new("Choose a camera:", cameras).prompt()?;

    let camera = ctx
        .get_camera(selected_camera_descriptor)
        .with_context(|| format!("Could not fetch the selected camera"))?;

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

        if !delete_confirmed {
            anyhow::bail!("No action performed, since delete flag was enabled, but not confirmed")
        }
    }

    let show_progress = match args.verbosity.log_level() {
        Some(log::Level::Info) | Some(log::Level::Debug) | Some(log::Level::Trace) => false,
        _ => true,
    };

    let file_count = if show_progress {
        Some(camera.count_files("/")?)
    } else {
        None
    };

    let pb = if show_progress && file_count.is_some() {
        let total_files = file_count.as_ref().unwrap().total_files;
        let progress_bar = if total_files > 0 {
            let progress_bar = ProgressBar::new(total_files);
            progress_bar.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files ({percent}%)")
                    .expect("Invalid progress bar template")
                    .progress_chars("█▉▊▋▌▍▎▏  ")
            );
            progress_bar
        } else {
            // do nothing if there are no files to process
            ProgressBar::hidden()
        };

        progress_bar
    } else {
        ProgressBar::hidden()
    };

    let progress_callback = {
        let pb_clone = pb.clone();
        move |current: u64, _total: u64| {
            pb_clone.set_position(current);
        }
    };

    // TODO: make it handle ctrlc properly
    let result =
        camera.move_all_files_with_callback("/", output_path, delete_confirmed, progress_callback);

    pb.finish_with_message("File transfer completed!");
    result?;

    Ok(())
}

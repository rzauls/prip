use anyhow::{Context, Result};
use clap::Parser;
use log::{info, trace};
use std::io::prelude::*;
use std::io::{self};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,
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

    let stdout = io::stdout();
    let mut stdout_handle = io::BufWriter::new(stdout.lock());

    trace!("environment initialized");

    let ctx = gphoto2::Context::new()?;
    let mut cameras: Vec<prip::Camera> = vec![];
    for cd in ctx.list_cameras().wait()? {
        trace!("detected {} on port {}", cd.model, cd.port);
        cameras.push(prip::Camera::new(cd));
    }

    let selected_camera = inquire::Select::new("Choose a camera:", cameras).prompt()?;

    let camera = gphoto2::Context::new()?
        .get_camera(&selected_camera.descriptor())
        .wait()
        .with_context(|| format!("could not get selected camera"))?;

    info!("autoselected camera summary:\n{}", camera.summary()?);
    info!("selected port: {}", camera.port_info()?.path());

    let camera_fs = camera.fs();

    let folders = prip::list_folders_recursive(&camera_fs, "/")?;

    writeln!(stdout_handle, "{}", folders)?;
    stdout_handle.flush()?;

    // TODO: add progress bar back in
    // TODO: check https://github.com/maxicarlos08/gphoto2-rs/blob/main/examples/camera_progress.rs
    // for progress with camera interactions

    // let pb = indicatif::ProgressBar::new(100);
    // for i in 0..100 {
    //     if !running.load(Ordering::SeqCst) {
    //         break;
    //     }
    //     pb.println(format!("[+] finished #{}", i));
    //     pb.inc(1);
    //     sleep(std::time::Duration::from_millis(5));
    // }
    // pb.finish_with_message("done");
    Ok(())
}

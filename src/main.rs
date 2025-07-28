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

    // let ctx = gphoto2::Context::new()?;
    // TODO: add the descriptors to a list so we can choose what camera to perform the operations
    // on
    // for gphoto2::list::CameraDescriptor { model, port } in ctx.list_cameras().wait()? {
    //     // writeln!(stdout_handle, "{} on port {}", model, port)?;
    //     info!("{} on port {}", model, port);
    // }

    // let paths = std::fs::read_dir(&args.path)
    //     .with_context(|| format!("could not read path`{}`", args.path.display()))?;

    let camera = gphoto2::Context::new()?
        .autodetect_camera()
        .wait()
        .with_context(|| format!("could not autodecetct camera"))?;

    info!("autoselected camera summary:\n{}", camera.summary()?);
    info!("selected port: {}", camera.port_info()?.path());

    let camera_fs = camera.fs();

    let folders = prip::list_directory_recursive(&camera_fs, "/");

    writeln!(stdout_handle, "{:?}", folders)?;
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

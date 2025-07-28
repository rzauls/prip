use anyhow::{Context, Result};
use clap::Parser;
use log::{info, warn};
use std::io::BufRead;
use std::io::{self, Write};
use std::thread::sleep;

#[derive(Parser)]
struct Cli {
    pattern: String,
    path: std::path::PathBuf,
    #[command(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,
}

fn main() -> Result<()> {
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
        match l {
            Ok(line_content) => {
                if line_content.contains(&args.pattern) {
                    writeln!(stdout_handle, "{}", line_content)?;
                };
            }
            Err(error) => {
                eprintln!("Error: {}", error);
            }
        }
    }

    let pb = indicatif::ProgressBar::new(100);
    for i in 0..100 {
        pb.println(format!("[+] finished #{}", i));
        pb.inc(1);
        sleep(std::time::Duration::from_millis(5));
    }
    pb.finish_with_message("done");
    warn!("app not done yet");
    Ok(())
}

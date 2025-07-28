use anyhow::{Context, Result};
use clap::Parser;
use log::{info, warn};
use std::io::{self, BufRead};
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
                find_matches(&line_content, &args.pattern, &mut stdout_handle)?;
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

fn find_matches(content: &str, pattern: &str, mut writer: impl std::io::Write) -> Result<()> {
    if content.contains(pattern) {
        writeln!(writer, "{}", content)?;
    };

    Ok(())
}

#[test]
fn check_if_the_world_exists() {
    assert_eq!("world", "world");
}

#[test]
fn find_a_match() {
    let mut result = Vec::new();
    let _ = find_matches("lorem ipsum", "lorem", &mut result);
    assert_eq!(result, b"lorem ipsum\n");
}

#[test]
fn dont_find_a_match() {
    let mut result: Vec<u8> = Vec::new();
    let _ = find_matches("lorem ipsum", "loreal", &mut result);
    assert_eq!(result, b"");
}

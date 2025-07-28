use anyhow::{Context, Result};
use clap::Parser;
use std::io::BufRead;

#[derive(Parser)]
struct Cli {
    pattern: String,
    path: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let f = std::fs::File::open(&args.path)
        .with_context(|| format!("could not read file `{}`", args.path.display()))?;
    let reader = std::io::BufReader::new(f);

    for l in reader.lines() {
        match l {
            Ok(line_content) => {
                if line_content.contains(&args.pattern) {
                    println!("{}", line_content);
                };
            }
            Err(error) => {
                println!("Error: {}", error);
            }
        }
    }
    Ok(())
}

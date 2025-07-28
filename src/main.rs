use clap::Parser;
use std::io::BufRead;

#[derive(Parser)]
struct Cli {
    pattern: String,
    path: std::path::PathBuf,
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let f = std::fs::File::open(args.path)?;
    let reader = std::io::BufReader::new(f);

    for l in reader.lines() {
        let line = String::from(l?);
        if line.contains(&args.pattern) {
            println!("{}", line);
        }
    }
    Ok(())
}

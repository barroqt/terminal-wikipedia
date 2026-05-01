use clap::Parser;
use std::error::Error;

#[derive(Parser)]
#[command(name = "terminal-wikipedia")]
struct Args {
    url: String,
}

fn validate_url(url: &str) -> Result<(), Box<dyn Error>> {
    if !url.contains(".wikipedia.org/wiki/") {
        return Err("Error: URL must be a Wikipedia article URL (e.g., https://en.wikipedia.org/wiki/Article)".into());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    validate_url(&args.url)?;
    println!("{}", args.url);
    Ok(())
}

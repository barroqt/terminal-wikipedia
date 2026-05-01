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

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&args.url)
        .header(
            "User-Agent",
            "terminal-wikipedia/0.1.0 (https://github.com/barroqt)",
        )
        .send()?;

    if !response.status().is_success() {
        return Err(format!(
            "HTTP error: {} {}",
            response.status().as_u16(),
            response
                .status()
                .canonical_reason()
                .unwrap_or("Unknown error")
        )
        .into());
    }

    let html = response.text()?;
    println!("{}", &html[0..100]);

    Ok(())
}

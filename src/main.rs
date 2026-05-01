use clap::Parser;
use scraper::{Html, Selector};
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

fn format_dom(element: scraper::ElementRef) -> String {
    const BOLD: &str = "\x1b[1m";
    const ITALIC: &str = "\x1b[3m";
    const RESET: &str = "\x1b[0m";

    let unwanted_classes = [
        "infobox", "sidebar", "navbox", "mw-editsection",
        "reflist", "references", "toc",
        "hatnote", "noprint", "mw-ref",
    ];

    fn should_remove(el: &scraper::ElementRef, unwanted: &[&str]) -> bool {
        for class in el.value().classes() {
            if unwanted.contains(&class) {
                return true;
            }
        }
        let tag = el.value().name();
        tag == "style" || tag == "script" || tag == "sup"
    }

    fn get_inner_text(el: scraper::ElementRef, unwanted: &[&str], bold: &str, italic: &str, reset: &str) -> String {
        let mut result = String::new();
        for child in el.children() {
            if let scraper::Node::Text(text) = child.value() {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    let clean = trimmed.replace(" .", ".").replace(" ,", ",").replace(" :", ":").replace(" )", ")").replace("( ", "(");
                    if !result.is_empty() && !result.ends_with(' ') && !result.ends_with('\n') && !clean.starts_with('.') && !clean.starts_with(',') && !clean.starts_with(':') && !clean.starts_with(')') && !clean.starts_with(';') {
                        result.push(' ');
                    }
                    result.push_str(&clean);
                }
            } else if let scraper::Node::Element(_) = child.value() {
                let child_ref = scraper::ElementRef::wrap(child).unwrap();
                if !should_remove(&child_ref, unwanted) {
                    let child_text = format_node(child_ref, unwanted, bold, italic, reset);
                    if !child_text.is_empty() {
                        if !result.is_empty() && !result.ends_with(' ') && !result.ends_with('\n') && !child_text.starts_with('.') && !child_text.starts_with(',') && !child_text.starts_with(':') && !child_text.starts_with(')') && !child_text.starts_with(';') {
                            result.push(' ');
                        }
                        result.push_str(&child_text);
                    }
                }
            }
        }
        if result.ends_with(" .") {
            result.truncate(result.len() - 2);
            result.push('.');
        }
        result
    }

    fn format_node(el: scraper::ElementRef, unwanted: &[&str], bold: &str, italic: &str, reset: &str) -> String {
        if should_remove(&el, unwanted) {
            return String::new();
        }

        let tag = el.value().name();
        let text = get_inner_text(el, unwanted, bold, italic, reset);

        match tag {
            "h1" => format!("####### {}\n\n", text),
            "h2" => format!("###### {}\n\n", text),
            "h3" => format!("##### {}\n\n", text),
            "h4" => format!("#### {}\n\n", text),
            "h5" => format!("### {}\n\n", text),
            "h6" => format!("## {}\n\n", text),
            "p" => format!("{}\n\n", text),
            "b" | "strong" => format!("{bold}{text}{reset}", bold = bold, text = text, reset = reset),
            "i" | "em" => format!("{italic}{text}{reset}", italic = italic, text = text, reset = reset),
            "a" => text,
            "ul" => {
                let mut result = String::new();
                for child in el.children() {
                    if let scraper::Node::Element(_) = child.value() {
                        let child_ref = scraper::ElementRef::wrap(child).unwrap();
                        if child_ref.value().name() == "li" {
                            let item_text = get_inner_text(child_ref, unwanted, bold, italic, reset).trim().to_string();
                            if !item_text.is_empty() {
                                result.push_str(&format!("• {}\n", item_text));
                            }
                        }
                    }
                }
                if !result.is_empty() {
                    result.push('\n');
                }
                result
            }
            "ol" => {
                let mut result = String::new();
                let mut idx = 1;
                for child in el.children() {
                    if let scraper::Node::Element(_) = child.value() {
                        let child_ref = scraper::ElementRef::wrap(child).unwrap();
                        if child_ref.value().name() == "li" {
                            let item_text = get_inner_text(child_ref, unwanted, bold, italic, reset).trim().to_string();
                            if !item_text.is_empty() {
                                result.push_str(&format!("{}. {}\n", idx, item_text));
                                idx += 1;
                            }
                        }
                    }
                }
                if !result.is_empty() {
                    result.push('\n');
                }
                result
            }
            "hr" => format!("{}\n\n", "-".repeat(80)),
            "br" => format!("\n"),
            _ => {
                if text.is_empty() {
                    let mut result = String::new();
                    for child in el.children() {
                        if let scraper::Node::Element(_) = child.value() {
                            let child_ref = scraper::ElementRef::wrap(child).unwrap();
                            if !should_remove(&child_ref, unwanted) {
                                result.push_str(&format_node(child_ref, unwanted, bold, italic, reset));
                            }
                        }
                    }
                    result
                } else {
                    text
                }
            }
        }
    }

    let mut result = String::new();
    for child in element.children() {
        if let scraper::Node::Element(_) = child.value() {
            let child_ref = scraper::ElementRef::wrap(child).unwrap();
            if !should_remove(&child_ref, &unwanted_classes) {
                result.push_str(&format_node(child_ref, &unwanted_classes, BOLD, ITALIC, RESET));
            }
        }
    }
    result.trim().to_string()
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
    let document = Html::parse_document(&html);

    let selector = Selector::parse("#mw-content-text .mw-parser-output").unwrap();
    let element = document.select(&selector).next();

    if let Some(element) = element {
        let text = format_dom(element);
        println!("{}", text);
    } else {
        return Err("Could not find article body".into());
    }

    Ok(())
}

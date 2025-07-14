#![allow(dead_code)]

mod article_extractor;
mod html_parser;
mod http_client;
mod rss_generator;
mod rss_item;

use article_extractor::{extract_article_content, markdown_to_html};
use chrono::{DateTime, Utc};
use html_parser::parse_articles_from_html;
use http_client::HttpClient;
use rss_generator::generate_rss;
use rss_item::RssItem;
use std::fs;

async fn fetch_and_generate_rss() -> Result<(), Box<dyn std::error::Error>> {
    let client = HttpClient::new();

    // Fetch the main page
    println!("Fetching Ledge.ai main page...");
    let main_page_html = client.fetch_url("https://ledge.ai/").await?;

    // Parse articles from the main page
    println!("Parsing articles from HTML...");
    let articles = parse_articles_from_html(&main_page_html)?;
    println!("Found {} articles", articles.len());

    let mut rss_items = Vec::new();

    // Fetch content for each article
    println!("Fetching article content for {} articles...", articles.len());
    for (i, article) in articles.iter().enumerate() {
        println!("Processing article {}/{}: {}", i + 1, articles.len(), article.title);
        
        match client.fetch_url(&article.url).await {
            Ok(article_html) => {
                if let Ok(markdown_content) = extract_article_content(&article_html) {
                    println!("  ✓ Extracted content ({} chars)", markdown_content.len());
                    let html_content = markdown_to_html(&markdown_content);
                    let pub_date = format_date(&article.date);

                    let rss_item = RssItem {
                        title: article.title.clone(),
                        link: article.url.clone(),
                        description: html_content,
                        pub_date,
                    };

                    rss_items.push(rss_item);
                } else {
                    eprintln!("  ✗ Failed to extract content from: {}", article.url);
                }
            }
            Err(e) => {
                eprintln!("  ✗ Failed to fetch article {}: {}", article.url, e);
            }
        }
    }

    // Generate RSS
    println!("Generating RSS feed with {} items...", rss_items.len());
    let rss_xml = generate_rss(rss_items)?;

    // Write to file
    fs::write("rss.xml", &rss_xml)?;

    println!("RSS feed generated successfully as 'rss.xml' ({} bytes)", rss_xml.len());
    Ok(())
}

fn format_date(date_str: &str) -> String {
    // Convert "2025/1/14 [TUE]" format to RFC 2822 format
    if let Some(date_part) = date_str.split(' ').next() {
        if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date_part, "%Y/%m/%d") {
            let datetime = parsed.and_hms_opt(12, 0, 0).unwrap();
            let utc_datetime: DateTime<Utc> = DateTime::from_naive_utc_and_offset(datetime, Utc);
            return utc_datetime.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        }
    }
    // Fallback to current time if parsing fails
    Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

#[tokio::main]
async fn main() {
    if let Err(e) = fetch_and_generate_rss().await {
        eprintln!("Error generating RSS feed: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date() {
        let date_str = "2025/1/14 [TUE]";
        let formatted = format_date(date_str);
        assert!(formatted.contains("14 Jan 2025"));
    }

    #[test]
    fn test_format_date_invalid() {
        let date_str = "invalid date";
        let formatted = format_date(date_str);
        // Should not panic and return some valid date format
        assert!(formatted.contains("GMT"));
    }
}

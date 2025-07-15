mod article_extractor;
mod html_parser;
mod http_client;
mod rss_generator;
mod rss_item;

use article_extractor::{extract_article_content, extract_article_date, markdown_to_html};
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

    // Temporary structure to hold article data with sortable date
    struct ArticleData {
        title: String,
        url: String,
        content: String,
        date_str: String, // ISO 8601 format for sorting
    }
    let mut article_data_list = Vec::new();

    // Fetch content for each article
    println!(
        "Fetching article content for {} articles...",
        articles.len()
    );
    for (i, article) in articles.iter().enumerate() {
        println!(
            "Processing article {}/{}: {}",
            i + 1,
            articles.len(),
            article.title
        );

        match client.fetch_url(&article.url).await {
            Ok(article_html) => {
                if let Ok(markdown_content) = extract_article_content(&article_html) {
                    println!("  ✓ Extracted content ({} chars)", markdown_content.len());
                    let html_content = markdown_to_html(&markdown_content);

                    // Try to extract actual publication date from article page
                    let actual_date = extract_article_date(&article_html);
                    let date_to_use = actual_date.as_deref().unwrap_or(&article.date);

                    if actual_date.is_some() {
                        println!("  ✓ Extracted publication date: {date_to_use}");
                    } else {
                        println!("  ! Using fallback date: {date_to_use}");
                    }

                    article_data_list.push(ArticleData {
                        title: article.title.clone(),
                        url: article.url.clone(),
                        content: html_content,
                        date_str: date_to_use.to_string(),
                    });
                } else {
                    eprintln!("  ✗ Failed to extract content from: {}", article.url);
                }
            }
            Err(e) => {
                eprintln!("  ✗ Failed to fetch article {}: {}", article.url, e);
            }
        }
    }

    // Sort articles by publication date (newest first)
    article_data_list.sort_by(|a, b| {
        // Parse ISO dates and compare (newest first, so b.cmp(a))
        let date_a = parse_iso_date(&a.date_str);
        let date_b = parse_iso_date(&b.date_str);
        date_b.cmp(&date_a)
    });

    // Convert to RssItem vector
    let rss_items: Vec<RssItem> = article_data_list
        .into_iter()
        .map(|data| RssItem {
            title: data.title,
            link: data.url,
            description: data.content,
            pub_date: parse_iso_date(&data.date_str),
        })
        .collect();

    // Generate RSS
    println!("Generating RSS feed with {} items...", rss_items.len());
    let rss_xml = generate_rss(rss_items)?;

    // Write to file
    fs::write("rss.xml", &rss_xml)?;

    println!(
        "RSS feed generated successfully as 'rss.xml' ({} bytes)",
        rss_xml.len()
    );
    Ok(())
}

fn parse_iso_date(date_str: &str) -> DateTime<Utc> {
    // Try ISO 8601 format first (e.g., "2025-07-14T07:50:00.000Z")
    if let Ok(parsed) = DateTime::parse_from_rfc3339(date_str) {
        return parsed.with_timezone(&Utc);
    }

    // Try standard ISO format without timezone
    if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S") {
        return DateTime::from_naive_utc_and_offset(parsed, Utc);
    }

    // Try "2025/1/14 [TUE]" format
    if let Some(date_part) = date_str.split(' ').next() {
        if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date_part, "%Y/%m/%d") {
            let datetime = parsed.and_hms_opt(12, 0, 0).unwrap();
            return DateTime::from_naive_utc_and_offset(datetime, Utc);
        }
    }

    // Fallback to current time if parsing fails
    Utc::now()
}

#[tokio::main]
async fn main() {
    if let Err(e) = fetch_and_generate_rss().await {
        eprintln!("Error generating RSS feed: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_parse_iso_date() {
        let date_str = "2025/1/14 [TUE]";
        let parsed = parse_iso_date(date_str);
        assert_eq!(parsed.format("%Y-%m-%d").to_string(), "2025-01-14");
    }

    #[test]
    fn test_parse_iso_date_invalid() {
        let date_str = "invalid date";
        let parsed = parse_iso_date(date_str);
        // Should not panic and return current time as fallback
        assert!(parsed.year() >= 2025);
    }

    #[test]
    fn test_parse_iso_date_iso8601() {
        let date_str = "2025-07-14T07:50:00.000Z";
        let parsed = parse_iso_date(date_str);
        assert_eq!(
            parsed.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2025-07-14 07:50:00"
        );
    }

    #[test]
    fn test_parse_iso_date_iso8601_no_timezone() {
        let date_str = "2025-07-14T07:50:00";
        let parsed = parse_iso_date(date_str);
        assert_eq!(
            parsed.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2025-07-14 07:50:00"
        );
    }
}

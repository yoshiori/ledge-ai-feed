use scraper::{Html, Selector};
use serde_json::Value;

#[derive(Debug, PartialEq)]
pub struct ArticleInfo {
    pub title: String,
    pub url: String,
    pub date: String,
}

pub fn parse_articles_from_html(
    html: &str,
) -> Result<Vec<ArticleInfo>, Box<dyn std::error::Error>> {
    // First try to extract from Nuxt.js __NUXT__ object
    println!("  Trying to extract from Nuxt data...");
    if let Ok(articles) = extract_from_nuxt_data(html) {
        if !articles.is_empty() {
            println!("  ✓ Found {} articles from Nuxt data", articles.len());
            return Ok(articles);
        }
    }
    println!("  No articles found in Nuxt data, trying static HTML...");

    // Fallback to static HTML parsing
    let static_articles = extract_from_static_html(html)?;
    println!(
        "  Found {} articles from static HTML",
        static_articles.len()
    );
    Ok(static_articles)
}

fn extract_from_nuxt_data(html: &str) -> Result<Vec<ArticleInfo>, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html);
    let script_selector = Selector::parse("script")?;

    for script_element in document.select(&script_selector) {
        let script_text = script_element.text().collect::<String>();

        // Try to find JSON data embedded in various formats
        if script_text.contains("articles") && (script_text.len() > 1000) {
            println!(
                "    Found large script with 'articles' keyword ({} chars)",
                script_text.len()
            );

            // Try to extract any JSON objects containing article data
            if let Some(articles) = extract_articles_from_any_json(&script_text) {
                if !articles.is_empty() {
                    println!(
                        "    ✓ Found {} articles from JSON extraction",
                        articles.len()
                    );
                    return Ok(articles);
                }
            }
        }
    }

    Err("Nuxt data not found".into())
}

fn extract_articles_from_any_json(script_text: &str) -> Option<Vec<ArticleInfo>> {
    let mut articles = Vec::new();

    // Look for patterns like "title":"something", "slug":"something"
    let title_pattern = regex::Regex::new(r#""title":"([^"]+)""#).ok()?;
    let slug_pattern = regex::Regex::new(r#""slug":"([^"]+)""#).ok()?;

    // Find all title matches
    let title_matches: Vec<_> = title_pattern.find_iter(script_text).collect();
    let _slug_matches: Vec<_> = slug_pattern.find_iter(script_text).collect();

    println!("    Found {} title matches", title_matches.len());

    // Pair up titles and slugs that are close to each other
    for title_match in title_matches {
        if let Some(title_caps) = title_pattern.captures(&script_text[title_match.range()]) {
            let title = title_caps.get(1)?.as_str();

            // Look for a slug within 500 characters of this title
            let search_start = title_match.start();
            let search_end = (title_match.end() + 500).min(script_text.len());
            let search_area = &script_text[search_start..search_end];

            if let Some(slug_caps) = slug_pattern.captures(search_area) {
                let slug = slug_caps.get(1)?.as_str();
                let url = format!("https://ledge.ai/articles/{}", slug);

                articles.push(ArticleInfo {
                    title: title.to_string(),
                    url,
                    date: "2025/01/14 [MON]".to_string(), // Fallback date
                });

                if articles.len() >= 10 {
                    // Limit to prevent too many duplicates
                    break;
                }
            }
        }
    }

    if articles.is_empty() {
        None
    } else {
        Some(articles)
    }
}

fn extract_articles_from_nuxt_json(
    json: &Value,
) -> Result<Vec<ArticleInfo>, Box<dyn std::error::Error>> {
    let mut articles = Vec::new();

    // Try different possible paths for article data
    let article_paths: Vec<Vec<&str>> = vec![
        vec!["data", "fetchNewestArticles"],
        vec!["data", "fetchFirstViewArticles"],
        vec!["state", "articles", "newest"],
        vec!["asyncData", "articles"],
    ];

    for path in &article_paths {
        if let Some(articles_data) = navigate_json_path(json, path) {
            if let Some(articles_array) = articles_data.as_array() {
                for article in articles_array {
                    if let Some(article_info) = parse_single_article(article) {
                        articles.push(article_info);
                    }
                }
                if !articles.is_empty() {
                    break;
                }
            }
        }
    }

    if articles.is_empty() {
        // Try to find articles in any array in the JSON
        find_articles_in_json_recursively(json, &mut articles);
    }

    Ok(articles)
}

fn navigate_json_path<'a>(json: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = json;
    for key in path {
        current = current.get(key)?;
    }
    Some(current)
}

fn parse_single_article(article: &Value) -> Option<ArticleInfo> {
    let title = article.get("title")?.as_str()?.to_string();

    // Try different possible fields for URL slug
    let slug = article
        .get("slug")
        .or_else(|| article.get("url"))
        .or_else(|| article.get("path"))
        .and_then(|v| v.as_str())?;

    let url = if slug.starts_with('/') {
        format!("https://ledge.ai{}", slug)
    } else if slug.starts_with("http") {
        slug.to_string()
    } else {
        format!("https://ledge.ai/articles/{}", slug)
    };

    // Try different possible fields for date
    let date = article
        .get("publishedAt")
        .or_else(|| article.get("createdAt"))
        .or_else(|| article.get("date"))
        .or_else(|| article.get("published_at"))
        .and_then(|v| v.as_str())
        .unwrap_or("2025/01/14 [MON]") // Fallback date
        .to_string();

    Some(ArticleInfo { title, url, date })
}

fn find_articles_in_json_recursively(json: &Value, articles: &mut Vec<ArticleInfo>) {
    match json {
        Value::Object(obj) => {
            for value in obj.values() {
                find_articles_in_json_recursively(value, articles);
            }
        }
        Value::Array(arr) => {
            // Check if this array contains article-like objects
            for item in arr {
                if let Some(article) = parse_single_article(item) {
                    articles.push(article);
                } else {
                    find_articles_in_json_recursively(item, articles);
                }
            }
        }
        _ => {}
    }
}

fn extract_from_static_html(html: &str) -> Result<Vec<ArticleInfo>, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html);
    let mut articles = Vec::new();

    // Try various selectors that might contain article links
    let selectors_to_try = [
        "a[href*='/articles/']",  // Any link containing '/articles/'
        "a[href^='/articles/']",  // Any link starting with '/articles/'
        ".article a",             // Links inside article elements
        ".post a",                // Links inside post elements
        "article a",              // Links inside article tags
        "h1 a, h2 a, h3 a, h4 a", // Links in headers
    ];

    for selector_str in &selectors_to_try {
        if let Ok(selector) = Selector::parse(selector_str) {
            println!("    Trying selector: {}", selector_str);
            let mut found_count = 0;

            for element in document.select(&selector) {
                if let Some(href) = element.value().attr("href") {
                    if href.contains("/articles/") || href.starts_with("/articles/") {
                        let title = element.text().collect::<String>().trim().to_string();

                        if !title.is_empty() && title.len() > 5 {
                            // Skip very short titles
                            let url = if href.starts_with('/') {
                                format!("https://ledge.ai{}", href)
                            } else {
                                href.to_string()
                            };

                            articles.push(ArticleInfo {
                                title,
                                url,
                                date: "2025/01/14 [MON]".to_string(), // Fallback date
                            });
                            found_count += 1;
                        }
                    }
                }
            }

            println!("      Found {} articles with this selector", found_count);
            if found_count > 0 {
                break; // Use the first selector that finds articles
            }
        }
    }

    // Remove duplicates based on URL
    articles.sort_by(|a, b| a.url.cmp(&b.url));
    articles.dedup_by(|a, b| a.url == b.url);

    Ok(articles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_articles_from_html_static() {
        let html = r#"
            <html>
                <body>
                    <div class="article-item">
                        <a href="/articles/test1">Test Article 1</a>
                    </div>
                    <div class="article-item">
                        <a href="/articles/test2">Test Article 2</a>
                    </div>
                </body>
            </html>
        "#;

        let articles = parse_articles_from_html(html).unwrap();
        assert_eq!(articles.len(), 2);

        assert_eq!(articles[0].title, "Test Article 1");
        assert_eq!(articles[0].url, "https://ledge.ai/articles/test1");
        assert_eq!(articles[0].date, "2025/01/14 [MON]"); // Default fallback date

        assert_eq!(articles[1].title, "Test Article 2");
        assert_eq!(articles[1].url, "https://ledge.ai/articles/test2");
        assert_eq!(articles[1].date, "2025/01/14 [MON]"); // Default fallback date
    }

    #[test]
    fn test_parse_articles_from_nuxt_data() {
        // Create a script with many repetitions to exceed 1000 chars and include "articles" keyword
        let script_content = format!(
            r#"const articles = {{"title":"世界最強AI「Grok 4」公開", "slug":"grok4_xai_ai_model_launch"}}, {{"title":"Hugging Face、「SmolLM 3」公開", "slug":"smollm3_128k_multilingual_reasoning_model"}}; {}"#,
            "// padding comment ".repeat(100) // Make it well over 1000 chars
        );

        let html = format!(
            r#"
            <html>
                <head>
                    <script>
                        {}
                    </script>
                </head>
                <body></body>
            </html>
        "#,
            script_content
        );

        let articles = parse_articles_from_html(&html).unwrap();
        assert_eq!(articles.len(), 2);

        assert_eq!(articles[0].title, "世界最強AI「Grok 4」公開");
        assert_eq!(
            articles[0].url,
            "https://ledge.ai/articles/grok4_xai_ai_model_launch"
        );
        assert_eq!(articles[0].date, "2025/01/14 [MON]"); // Fallback date

        assert_eq!(articles[1].title, "Hugging Face、「SmolLM 3」公開");
        assert_eq!(
            articles[1].url,
            "https://ledge.ai/articles/smollm3_128k_multilingual_reasoning_model"
        );
        assert_eq!(articles[1].date, "2025/01/14 [MON]"); // Fallback date
    }

    #[test]
    fn test_parse_single_article() {
        let article_json = serde_json::json!({
            "title": "Test Article",
            "slug": "test-article-slug",
            "publishedAt": "2025/01/14 [TUE]"
        });

        let article_info = parse_single_article(&article_json).unwrap();
        assert_eq!(article_info.title, "Test Article");
        assert_eq!(
            article_info.url,
            "https://ledge.ai/articles/test-article-slug"
        );
        assert_eq!(article_info.date, "2025/01/14 [TUE]");
    }

    #[test]
    fn test_parse_single_article_with_path() {
        let article_json = serde_json::json!({
            "title": "Test Article with Path",
            "path": "/articles/test-path",
            "createdAt": "2025/01/15 [WED]"
        });

        let article_info = parse_single_article(&article_json).unwrap();
        assert_eq!(article_info.title, "Test Article with Path");
        assert_eq!(article_info.url, "https://ledge.ai/articles/test-path");
        assert_eq!(article_info.date, "2025/01/15 [WED]");
    }

    #[test]
    fn test_extract_from_nuxt_data_not_found() {
        let html = r#"
            <html>
                <head>
                    <script>
                        console.log("No NUXT data here");
                    </script>
                </head>
                <body></body>
            </html>
        "#;

        let result = extract_from_nuxt_data(html);
        assert!(result.is_err());
    }
}

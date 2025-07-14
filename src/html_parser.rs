use scraper::{Html, Selector};

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

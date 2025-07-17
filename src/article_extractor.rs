use pulldown_cmark::{html, Parser};
use regex::Regex;
use scraper::{Html, Selector};

pub fn extract_article_content(html: &str) -> Result<String, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html);

    // Try multiple approaches to extract content

    // 1. Try to extract from script tags with various patterns
    if let Ok(content) = extract_from_script_tags(html) {
        if content.len() > 100 {
            // Final cleanup: remove any remaining {target="_blank"} patterns
            let cleaned_content = content.replace(r#"{target="_blank"}"#, "");
            return Ok(cleaned_content);
        }
    }

    // 2. Try to extract from standard HTML content areas
    if let Ok(content) = extract_from_html_content(&document) {
        // Final cleanup: remove any remaining {target="_blank"} patterns
        let cleaned_content = content.replace(r#"{target="_blank"}"#, "");
        return Ok(cleaned_content);
    }

    Err("Article content not found".into())
}

fn extract_from_script_tags(html: &str) -> Result<String, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html);
    let script_selector = Selector::parse("script")?;

    for script_element in document.select(&script_selector) {
        let script_text = script_element.text().collect::<String>();

        // Try different patterns
        if script_text.contains("__INITIAL_STATE__")
            || script_text.contains("__NUXT__")
            || script_text.len() > 5000
        {
            // Look for content in various formats
            if let Some(content) = extract_content_from_script(&script_text) {
                return Ok(content);
            }
        }
    }

    Err("No content found in scripts".into())
}

fn extract_content_from_script(script_text: &str) -> Option<String> {
    // Try to find various content patterns in Nuxt.js data - more flexible patterns
    let content_patterns = [
        // More flexible patterns for Ledge.ai
        r#""body":"([^"]{300,}?)""#,     // Body field, at least 300 chars
        r#""content":"([^"]{300,}?)""#,  // Content field, at least 300 chars
        r#""markdown":"([^"]{300,}?)""#, // Markdown content, at least 300 chars
        r#""text":"([^"]{300,}?)""#,     // Text field, at least 300 chars
        r#"article.*?content.*?:"([^"]{300,}?)""#, // Article with content
        r#"post.*?body.*?:"([^"]{300,}?)""#, // Post with body
        r#"contents.*?body.*?:"([^"]{300,}?)""#, // Contents array with body
    ];

    for pattern in &content_patterns {
        if let Ok(regex) = Regex::new(pattern) {
            if let Some(captures) = regex.captures(script_text) {
                if let Some(content_match) = captures.get(1) {
                    let content = content_match.as_str();

                    // Clean up escaped characters and remove {target="_blank"} patterns
                    let cleaned = content
                        .replace("\\n", "\n")
                        .replace("\\r", "\r")
                        .replace("\\t", "\t")
                        .replace("\\\"", "\"")
                        .replace("\\/", "/")
                        .replace(r#"{target="_blank"}"#, "");

                    // More lenient filtering - accept any substantial content
                    if cleaned.len() > 300 {
                        return Some(cleaned);
                    }
                }
            }
        }
    }

    None
}

fn extract_date_from_nuxt_script(script_text: &str) -> Option<String> {
    // Look for __NUXT__ object with article date information
    if script_text.contains("__NUXT__") {
        // Try to find the main article by looking for the structure that contains title, slug, and multiple date fields
        // This is likely to be the main article object (not just tags or other metadata)
        let main_article_patterns = [
            // Pattern 1: Look for attributes with title, slug, and dates (most specific)
            r#"attributes:\{[^}]*?title:[^}]*?slug:[^}]*?publishedAt:"([0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}[^"]*)"[^}]*?\}"#,
            // Pattern 2: Look for attributes with title and dates
            r#"attributes:\{[^}]*?title:[^}]*?publishedAt:"([0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}[^"]*)"[^}]*?\}"#,
            // Pattern 3: Look for the pattern that includes scheduled_at (indicates main article)
            r#"attributes:\{[^}]*?scheduled_at:[^}]*?publishedAt:"([0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}[^"]*)"[^}]*?\}"#,
        ];

        for pattern in &main_article_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(captures) = regex.captures(script_text) {
                    if let Some(date_match) = captures.get(1) {
                        let date = date_match.as_str();
                        return Some(date.to_string());
                    }
                }
            }
        }

        // Fallback: look for any date fields in the script
        let date_patterns = [
            r#"publishedAt:"([0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}[^"]*)"#,
            r#"scheduled_at:"([0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}[^"]*)"#,
            r#"createdAt:"([0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}[^"]*)"#,
            r#"updatedAt:"([0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}[^"]*)"#,
        ];

        for pattern in &date_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(captures) = regex.captures(script_text) {
                    if let Some(date_match) = captures.get(1) {
                        let date = date_match.as_str();
                        // Return the first valid date found
                        return Some(date.to_string());
                    }
                }
            }
        }
    }

    None
}

fn extract_date_from_nuxt_object(html: &str) -> Option<String> {
    // Look for __NUXT__ object in script tags
    let document = Html::parse_document(html);
    let script_selector = Selector::parse("script").ok()?;

    for script_element in document.select(&script_selector) {
        let script_text = script_element.text().collect::<String>();

        // Check if this script contains __NUXT__ object
        if script_text.contains("__NUXT__") {
            if let Some(date) = extract_date_from_nuxt_script(&script_text) {
                return Some(date);
            }
        }
    }

    None
}

fn extract_from_html_content(document: &Html) -> Result<String, Box<dyn std::error::Error>> {
    // Try Ledge.ai specific content selectors first
    let content_selectors = [
        ".article-body",    // Ledge.ai article body
        ".post-body",       // Post body
        ".content-body",    // Content body
        "article .content", // Article content
        ".entry-content",   // Entry content
        "main article",     // Main article
        "article",          // Generic article
        ".post-content",    // Post content
        "main",             // Main content area
        "[role='main']",    // Main role
    ];

    for selector_str in &content_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let content = element.text().collect::<Vec<_>>().join(" ");

                // More lenient filtering - just check basic length
                if content.len() > 300 {
                    // Basic cleanup for common UI elements and remove {target="_blank"} patterns
                    let cleaned = content
                        .replace("クリップする", "")
                        .replace("アクセスランキング", "")
                        .replace("関連記事", "")
                        .replace("人気のタグ", "")
                        .replace("FOLLOW US", "")
                        .replace(r#"{target="_blank"}"#, "");

                    return Ok(format!("# Article Content\n\n{}", cleaned.trim()));
                }
            }
        }
    }

    Err("No content found in HTML".into())
}

pub fn extract_article_date(html: &str) -> Option<String> {
    let document = Html::parse_document(html);

    // Try multiple strategies to extract publication date

    // 1. Try __NUXT__ object first (most reliable for Ledge.ai)
    if let Some(date) = extract_date_from_nuxt_object(html) {
        return Some(date);
    }

    // 2. Try meta tags
    if let Some(date) = extract_date_from_meta_tags(&document) {
        return Some(date);
    }

    // 3. Try time elements
    if let Some(date) = extract_date_from_time_elements(&document) {
        return Some(date);
    }

    // 4. Try JSON-LD structured data
    if let Some(date) = extract_date_from_json_ld(html) {
        return Some(date);
    }

    None
}

fn extract_date_from_meta_tags(document: &Html) -> Option<String> {
    let meta_selectors = [
        "meta[property=\"article:published_time\"]",
        "meta[name=\"article:published_time\"]",
        "meta[property=\"og:published_time\"]",
        "meta[name=\"publishedDate\"]",
        "meta[name=\"publication-date\"]",
        "meta[name=\"date\"]",
    ];

    for selector_str in &meta_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                if let Some(content) = element.value().attr("content") {
                    return Some(content.to_string());
                }
            }
        }
    }

    None
}

fn extract_date_from_time_elements(document: &Html) -> Option<String> {
    let time_selectors = [
        "time[datetime]",
        "time[pubdate]",
        ".published-date time",
        ".article-date time",
        ".post-date time",
    ];

    for selector_str in &time_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                if let Some(datetime) = element.value().attr("datetime") {
                    return Some(datetime.to_string());
                }
                // Fallback to text content if no datetime attribute
                let text_content = element.text().collect::<String>().trim().to_string();
                if !text_content.is_empty() && text_content.len() > 8 {
                    return Some(text_content);
                }
            }
        }
    }

    None
}

fn extract_date_from_json_ld(html: &str) -> Option<String> {
    let json_ld_pattern = Regex::new(
        r#"(?s)<script[^>]*type=["']application/ld\+json["'][^>]*>\s*(.*?)\s*</script>"#,
    )
    .ok()?;

    for capture in json_ld_pattern.captures_iter(html) {
        if let Some(json_content) = capture.get(1) {
            let json_str = json_content.as_str().trim();
            if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(json_str) {
                // Try different paths for publication date in JSON-LD
                let date_paths = [
                    "datePublished",
                    "dateCreated",
                    "dateModified",
                    "publishedDate",
                ];

                for path in &date_paths {
                    if let Some(date_value) = json_data.get(path) {
                        if let Some(date_str) = date_value.as_str() {
                            return Some(date_str.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

pub fn markdown_to_html(markdown: &str) -> String {
    // First, preprocess markdown to handle custom extensions and content filtering
    let preprocessed = preprocess_markdown_content(markdown);

    let parser = Parser::new(&preprocessed);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

/// Integrated markdown preprocessing that handles all content filtering
/// This replaces both content_filter::filter_content and preprocess_markdown_extensions
pub fn preprocess_markdown_content(markdown: &str) -> String {
    use once_cell::sync::Lazy;
    use regex::Regex;

    // Processing markdown content with integrated filtering

    // Compile all regex patterns once using Lazy static initialization
    static SMALL_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r":::small[\s\S]*?:::").unwrap());
    static BOX_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r":::box[\s\S]*?:::").unwrap());
    // Pattern to remove {target="_blank"} with both ASCII and Unicode quotes
    // Handles: ASCII quotes " and Unicode smart quotes " " (U+201C and U+201D)
    static TARGET_BLANK_PATTERN: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"\{target=[""\u{201C}]_blank[""\u{201D}]\}"#).unwrap());

    // Apply all filtering in sequence, minimizing string allocations
    let result = SMALL_PATTERN.replace_all(markdown, "");
    let result = BOX_PATTERN.replace_all(&result, "");

    // Apply target="_blank" pattern removal
    let result = TARGET_BLANK_PATTERN.replace_all(&result, "");

    result.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_article_content() {
        let html = r###"
            <html>
                <head>
                    <script>
                        var data = {"body": "# Test Article\\n\\nThis is a test article with sufficient content to pass the length check. This content is long enough to be considered substantial and will be extracted successfully by our content extraction algorithm. Additional content to ensure it meets the 300 character minimum requirement for our regex pattern matching system."};
                    </script>
                </head>
                <body>
                    <div>Navigation</div>
                    <main>Article content with enough text to pass the 300 character minimum requirement for substantial content extraction in our HTML parsing function. This main content area contains substantial text that should be extracted when script extraction fails. Additional text to ensure length requirements are met.</main>
                </body>
            </html>
        "###;

        let content = extract_article_content(html).unwrap();
        assert!(content.len() > 100); // Just check that we got substantial content
        assert!(content.contains("Test Article") || content.contains("Article content"));
    }

    #[test]
    fn test_markdown_to_html() {
        let markdown = "# Title\n\nThis is **bold** text.";
        let html = markdown_to_html(markdown);
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_markdown_to_html_with_target_blank() {
        let markdown = r#"Check this [link](https://example.com){target="_blank"} for more info."#;
        let html = markdown_to_html(markdown);
        // Should not contain the {target="_blank"} attribute
        assert!(!html.contains(r#"{target="_blank"}"#));
        // Should contain the link without the attribute
        assert!(html.contains(r#"<a href="https://example.com">link</a>"#));
    }

    #[test]
    fn test_preprocess_markdown_content_removes_target_blank() {
        let markdown = r#"Here is a [link](https://example.com){target="_blank"} and another [link](https://other.com){target="_blank"}."#;
        let result = preprocess_markdown_content(markdown);
        let expected =
            r#"Here is a [link](https://example.com) and another [link](https://other.com)."#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_preprocess_markdown_content_handles_standalone_target_blank() {
        // Test standalone {target="_blank"} patterns that are not part of markdown links
        let markdown = r#"Some text [link](https://example.com){target="_blank"} and then {target="_blank"} more text"#;
        let result = preprocess_markdown_content(markdown);
        let expected = r#"Some text [link](https://example.com) and then  more text"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_preprocess_markdown_content_handles_mixed_patterns() {
        // Test mixed patterns: links with target="_blank" and standalone target="_blank"
        let markdown = r#"Check [this](https://example.com){target="_blank"} and{target="_blank"}that{target="_blank"}"#;
        let result = preprocess_markdown_content(markdown);
        let expected = r#"Check [this](https://example.com) andthat"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_preprocess_markdown_content_integrates_all_filtering() {
        // Test that the new integrated function handles all patterns
        let markdown = r#"
Some content here.

:::small
画像の出典：GPT-4oによりLedge.aiが生成
:::

More content with a [link](https://example.com){target="_blank"} here.

:::box
関連記事：NECとさくらインターネットが協業、生成AIプラットフォーム分野でcotomiを拡張
:::

Text with{target="_blank"}standalone pattern.

Final content.
        "#;
        let result = preprocess_markdown_content(markdown);
        let expected = r#"
Some content here.



More content with a [link](https://example.com) here.



Text withstandalone pattern.

Final content.
        "#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_preprocess_markdown_content_removes_small_patterns() {
        let markdown = "Some text :::small\n画像の出典：GPT-4oによりLedge.aiが生成\n::: more text";
        let result = preprocess_markdown_content(markdown);
        let expected = "Some text  more text";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_preprocess_markdown_content_removes_box_patterns() {
        let markdown = "Some text :::box\n関連記事：NECとさくらインターネットが協業\n::: more text";
        let result = preprocess_markdown_content(markdown);
        let expected = "Some text  more text";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_article_date_from_meta_tags() {
        let html = r###"
            <html>
                <head>
                    <meta property="article:published_time" content="2025-07-14T07:50:00.000Z">
                    <title>Test Article</title>
                </head>
                <body>
                    <h1>Test Article</h1>
                    <p>Content</p>
                </body>
            </html>
        "###;

        let date = extract_article_date(html);
        assert!(date.is_some());
        assert_eq!(date.unwrap(), "2025-07-14T07:50:00.000Z");
    }

    #[test]
    fn test_extract_article_date_from_time_element() {
        let html = r###"
            <html>
                <head>
                    <title>Test Article</title>
                </head>
                <body>
                    <h1>Test Article</h1>
                    <time datetime="2025-07-14T07:50:00.000Z">July 14, 2025</time>
                    <p>Content</p>
                </body>
            </html>
        "###;

        let date = extract_article_date(html);
        assert!(date.is_some());
        assert_eq!(date.unwrap(), "2025-07-14T07:50:00.000Z");
    }

    #[test]
    fn test_extract_article_date_from_json_ld() {
        let html = r###"
            <html>
                <head>
                    <title>Test Article</title>
                    <script type="application/ld+json">
                    {
                        "@context": "https://schema.org",
                        "@type": "Article",
                        "headline": "Test Article",
                        "datePublished": "2025-07-14T07:50:00.000Z",
                        "author": "Test Author"
                    }
                    </script>
                </head>
                <body>
                    <h1>Test Article</h1>
                    <p>Content</p>
                </body>
            </html>
        "###;

        let date = extract_article_date(html);
        assert!(date.is_some());
        assert_eq!(date.unwrap(), "2025-07-14T07:50:00.000Z");
    }

    #[test]
    fn test_extract_article_date_no_date_found() {
        let html = r###"
            <html>
                <head>
                    <title>Test Article</title>
                </head>
                <body>
                    <h1>Test Article</h1>
                    <p>Content without any date information</p>
                </body>
            </html>
        "###;

        let date = extract_article_date(html);
        assert!(date.is_none());
    }

    #[test]
    fn test_extract_article_date_og_property() {
        let html = r###"
            <html>
                <head>
                    <meta property="og:published_time" content="2025-07-15T10:30:00.000Z">
                    <title>Test Article with OG tags</title>
                </head>
                <body>
                    <h1>Test Article</h1>
                    <p>Content</p>
                </body>
            </html>
        "###;

        let date = extract_article_date(html);
        assert!(date.is_some());
        assert_eq!(date.unwrap(), "2025-07-15T10:30:00.000Z");
    }

    #[test]
    fn test_extract_date_from_nuxt_object() {
        let html = r###"
            <html>
                <head>
                    <title>Test Article</title>
                </head>
                <body>
                    <script>
                        window.__NUXT__={attributes:{title:"Test Article",slug:"test-article",publishedAt:"2025-07-13T04:50:00.014Z",createdAt:"2025-07-11T09:46:20.383Z",updatedAt:"2025-07-13T04:50:00.087Z",scheduled_at:"2025-07-13T04:50:00.000Z"}};
                    </script>
                    <h1>Test Article</h1>
                    <p>Content</p>
                </body>
            </html>
        "###;

        let date = extract_article_date(html);
        assert!(date.is_some());
        assert_eq!(date.unwrap(), "2025-07-13T04:50:00.014Z");
    }

    #[test]
    fn test_extract_date_from_nuxt_script() {
        let script_text = r###"
            window.__NUXT__={attributes:{title:"Test Article",slug:"test-article",publishedAt:"2025-07-13T04:50:00.014Z",createdAt:"2025-07-11T09:46:20.383Z",updatedAt:"2025-07-13T04:50:00.087Z",scheduled_at:"2025-07-13T04:50:00.000Z"}};
        "###;

        let date = extract_date_from_nuxt_script(script_text);
        assert!(date.is_some());
        assert_eq!(date.unwrap(), "2025-07-13T04:50:00.014Z");
    }

    #[test]
    fn test_extract_date_from_nuxt_script_with_multiple_articles() {
        let script_text = r###"
            window.__NUXT__={
                tags:[
                    {id:1,attributes:{name:"tag1",publishedAt:"2024-01-01T00:00:00.000Z"}},
                    {id:2,attributes:{name:"tag2",publishedAt:"2024-02-01T00:00:00.000Z"}}
                ],
                article:{id:123,attributes:{title:"Main Article",slug:"main-article",scheduled_at:"2025-07-13T04:50:00.000Z",publishedAt:"2025-07-13T04:50:00.014Z",createdAt:"2025-07-11T09:46:20.383Z"}}
            };
        "###;

        let date = extract_date_from_nuxt_script(script_text);
        assert!(date.is_some());
        assert_eq!(date.unwrap(), "2025-07-13T04:50:00.014Z");
    }

    #[test]
    fn test_pulldown_cmark_with_special_characters() {
        // Test if pulldown-cmark itself introduces {target="_blank"} patterns
        let markdown =
            r#"Check this [link](https://www.nedo.go.jp/koubo/CD3_100397.html) for more info."#;
        let html = markdown_to_html(markdown);
        // Should not contain {target="_blank"} patterns
        assert!(!html.contains(r#"{target="_blank"}"#));
        // Should be a normal link
        assert!(html.contains(r#"<a href="https://www.nedo.go.jp/koubo/CD3_100397.html">link</a>"#));
    }

    #[test]
    fn test_markdown_with_japanese_and_encoded_urls() {
        // Test case similar to what we might see in the RSS problem
        let markdown = r#"今回の公募には43件が応募し、最終的に24件が採択された。楽天グループと野村総合研究所（NRI）を含む13件が新規採択で、第1期・第2期に採択されていなかった新顔の参画が加速している。"#;
        let html = markdown_to_html(markdown);
        // Should not contain any {target="_blank"} patterns
        assert!(!html.contains(r#"{target="_blank"}"#));
    }

    #[test]
    fn test_preprocess_markdown_content_handles_unicode_escaped_urls() {
        // Test the actual pattern found in the GENIAC article - simplified approach
        let markdown = r#"NEDOは、生成AI開発支援プロジェクト「GENIAC」の第3期採択結果を[発表](https:\u002F\u002Fwww.nedo.go.jp\u002Fkoubo\u002FCD3_100397.html){target="_blank"}した。"#;
        let result = preprocess_markdown_content(markdown);
        let expected = r#"NEDOは、生成AI開発支援プロジェクト「GENIAC」の第3期採択結果を[発表](https:\u002F\u002Fwww.nedo.go.jp\u002Fkoubo\u002FCD3_100397.html)した。"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_preprocess_markdown_content_handles_multiple_unicode_escapes() {
        // Test multiple unicode escapes in one URL - simplified approach just removes {target="_blank"}
        let markdown = r#"Check this [link](https:\u002F\u002Fexample.com\u002Fpath\u002Fto\u002Ffile){target="_blank"} for more info."#;
        let result = preprocess_markdown_content(markdown);
        let expected = r#"Check this [link](https:\u002F\u002Fexample.com\u002Fpath\u002Fto\u002Ffile) for more info."#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_preprocess_markdown_content_exact_geniac_pattern() {
        // Test the exact pattern found in the GENIAC article
        let markdown = r#"NEDOは、生成AI開発支援プロジェクト「GENIAC」の第3期採択結果を[発表](https:\u002F\u002Fwww.nedo.go.jp\u002Fkoubo\u002FCD3_100397.html){target="_blank"}した。"#;
        let result = preprocess_markdown_content(markdown);
        let expected = r#"NEDOは、生成AI開発支援プロジェクト「GENIAC」の第3期採択結果を[発表](https:\u002F\u002Fwww.nedo.go.jp\u002Fkoubo\u002FCD3_100397.html)した。"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_link_target_blank_regex_directly() {
        use regex::Regex;
        let pattern = Regex::new(r#"(\]\([^)]*\))[ ]*\{target="_blank"\}"#).unwrap();
        let test_text = r#"[発表](https:\u002F\u002Fwww.nedo.go.jp\u002Fkoubo\u002FCD3_100397.html){target="_blank"}"#;
        let result = pattern.replace_all(test_text, "$1");
        assert_eq!(
            result,
            r#"[発表](https:\u002F\u002Fwww.nedo.go.jp\u002Fkoubo\u002FCD3_100397.html)"#
        );
    }
}

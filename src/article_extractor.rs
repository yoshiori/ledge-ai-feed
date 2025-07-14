use pulldown_cmark::{html, Parser};
use scraper::{Html, Selector};

fn safe_substring(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        return s;
    }
    
    // Find a safe character boundary
    for i in (0..=max_len).rev() {
        if s.is_char_boundary(i) {
            return &s[..i];
        }
    }
    
    s
}

pub fn extract_article_content(html: &str) -> Result<String, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html);
    
    // Try multiple approaches to extract content
    
    // 1. Try to extract from script tags with various patterns
    if let Ok(content) = extract_from_script_tags(html) {
        if content.len() > 100 {
            return Ok(content);
        }
    }
    
    // 2. Try to extract from standard HTML content areas
    if let Ok(content) = extract_from_html_content(&document) {
        return Ok(content);
    }

    Err("Article content not found".into())
}

fn extract_from_script_tags(html: &str) -> Result<String, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html);
    let script_selector = Selector::parse("script")?;

    for script_element in document.select(&script_selector) {
        let script_text = script_element.text().collect::<String>();
        
        // Try different patterns
        if script_text.contains("__INITIAL_STATE__") || script_text.contains("__NUXT__") || script_text.len() > 5000 {
            
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
        r#""body":"([^"]{300,}?)""#,                        // Body field, at least 300 chars
        r#""content":"([^"]{300,}?)""#,                     // Content field, at least 300 chars  
        r#""markdown":"([^"]{300,}?)""#,                    // Markdown content, at least 300 chars
        r#""text":"([^"]{300,}?)""#,                        // Text field, at least 300 chars
        r#"article.*?content.*?:"([^"]{300,}?)""#,          // Article with content
        r#"post.*?body.*?:"([^"]{300,}?)""#,                // Post with body
        r#"contents.*?body.*?:"([^"]{300,}?)""#,            // Contents array with body
    ];
    
    for pattern in &content_patterns {
        if let Ok(regex) = regex::Regex::new(pattern) {
            if let Some(captures) = regex.captures(script_text) {
                if let Some(content_match) = captures.get(1) {
                    let content = content_match.as_str();
                    
                    // Clean up escaped characters
                    let cleaned = content
                        .replace("\\n", "\n")
                        .replace("\\r", "\r")
                        .replace("\\t", "\t")
                        .replace("\\\"", "\"")
                        .replace("\\/", "/");
                    
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

fn extract_from_html_content(document: &Html) -> Result<String, Box<dyn std::error::Error>> {
    // Try Ledge.ai specific content selectors first
    let content_selectors = [
        ".article-body",        // Ledge.ai article body
        ".post-body",          // Post body
        ".content-body",       // Content body
        "article .content",    // Article content
        ".entry-content",      // Entry content
        "main article",        // Main article
        "article",             // Generic article
        ".post-content",       // Post content
        "main",                // Main content area
        "[role='main']",       // Main role
    ];
    
    for selector_str in &content_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let content = element.text().collect::<Vec<_>>().join(" ");
                
                // More lenient filtering - just check basic length
                if content.len() > 300 {
                    
                    // Basic cleanup for common UI elements but still return content
                    let cleaned = content
                        .replace("クリップする", "")
                        .replace("アクセスランキング", "")
                        .replace("関連記事", "")
                        .replace("人気のタグ", "")
                        .replace("FOLLOW US", "");
                    
                    return Ok(format!("# Article Content\n\n{}", cleaned.trim()));
                }
            }
        }
    }
    
    Err("No content found in HTML".into())
}

pub fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
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
                        window.__INITIAL_STATE__ = {
                            "article": {
                                "content": "# Test Article\\n\\nThis is a test article with sufficient content to pass the length check. This content is long enough to be considered substantial and will be extracted successfully by our content extraction algorithm."
                            }
                        };
                    </script>
                </head>
                <body>
                    <div>Navigation</div>
                    <main>Article content with enough text to pass the 200 character minimum requirement for substantial content extraction in our HTML parsing function.</main>
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
}

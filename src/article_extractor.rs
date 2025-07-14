use pulldown_cmark::{html, Parser};
use scraper::{Html, Selector};

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
    // Try to find markdown or HTML content patterns
    let content_patterns = [
        r#""content":"([^"]{100,})""#,  // At least 100 chars
        r#""body":"([^"]{100,})""#,
        r#""markdown":"([^"]{100,})""#,
        r#""text":"([^"]{100,})""#,
    ];
    
    for pattern in &content_patterns {
        if let Ok(regex) = regex::Regex::new(pattern) {
            if let Some(captures) = regex.captures(script_text) {
                if let Some(content_match) = captures.get(1) {
                    let content = content_match.as_str();
                    return Some(content.replace("\\n", "\n").replace("\\\"", "\""));
                }
            }
        }
    }
    
    None
}

fn extract_from_html_content(document: &Html) -> Result<String, Box<dyn std::error::Error>> {
    // Try common content selectors
    let content_selectors = [
        "article",
        ".article-content", 
        ".post-content",
        ".content",
        "main",
        ".entry-content",
        "[role='main']",
    ];
    
    for selector_str in &content_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let content = element.text().collect::<Vec<_>>().join(" ");
                if content.len() > 200 { // Only return substantial content
                    return Ok(format!("# Article Content\n\n{}", content.trim()));
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

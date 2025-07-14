use pulldown_cmark::{html, Parser};
use scraper::{Html, Selector};
use serde_json::Value;

pub fn extract_article_content(html: &str) -> Result<String, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html);
    let script_selector = Selector::parse("script")?;

    for script_element in document.select(&script_selector) {
        let script_text = script_element.text().collect::<String>();
        if script_text.contains("__INITIAL_STATE__") {
            if let Some(json_start) = script_text.find('{') {
                if let Some(json_end) = script_text.rfind('}') {
                    let json_str = &script_text[json_start..=json_end];
                    let cleaned_json = json_str.replace("};", "}");
                    
                    if let Ok(json_value) = serde_json::from_str::<Value>(&cleaned_json) {
                        if let Some(content) = json_value
                            .get("article")
                            .and_then(|a| a.get("contents"))
                            .and_then(|c| c.get(0))
                            .and_then(|item| item.get("content"))
                            .and_then(|content| content.as_str())
                        {
                            return Ok(content.replace("\\n", "\n"));
                        }
                    }
                }
            }
        }
    }

    Err("Article content not found".into())
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
                                "contents": [
                                    {
                                        "content": "# Test Article\\n\\nThis is test content."
                                    }
                                ]
                            }
                        };
                    </script>
                </head>
                <body>
                    <div>Navigation</div>
                    <main>Article content</main>
                </body>
            </html>
        "###;

        let content = extract_article_content(html).unwrap();
        assert_eq!(content, "# Test Article\n\nThis is test content.");
    }

    #[test]
    fn test_markdown_to_html() {
        let markdown = "# Title\n\nThis is **bold** text.";
        let html = markdown_to_html(markdown);
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }
}
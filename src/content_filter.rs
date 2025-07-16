use once_cell::sync::Lazy;
use regex::Regex;

// Compile regex patterns once using Lazy static initialization
static SMALL_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r":::small[\s\S]*?:::").unwrap());
static BOX_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r":::box[\s\S]*?:::").unwrap());
static TARGET_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\{target="_blank"\}"#).unwrap());
static TARGET_ENCODED_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\{target=&quot;_blank&quot;\}"#).unwrap());
static TARGET_DOUBLE_ENCODED_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\{target=&amp;quot;_blank&amp;quot;\}"#).unwrap());

/// Content filter for cleaning up RSS article content
/// Removes unwanted patterns like image attributions, related article boxes, and link attributes
pub fn filter_content(content: &str) -> String {
    // Chain replacements to minimize allocations
    let result = SMALL_PATTERN.replace_all(content, "").to_string();
    let result = BOX_PATTERN.replace_all(&result, "").to_string();
    let result = TARGET_PATTERN.replace_all(&result, "").to_string();
    let result = TARGET_ENCODED_PATTERN.replace_all(&result, "").to_string();
    let result = TARGET_DOUBLE_ENCODED_PATTERN
        .replace_all(&result, "")
        .to_string();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_small_image_attribution() {
        let content = "Some text :::small\n画像の出典：GPT-4oによりLedge.aiが生成\n::: more text";
        let expected = "Some text  more text";
        let result = filter_content(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_remove_box_related_articles() {
        let content = "Some text :::box\n関連記事：NECとさくらインターネットが協業、生成AIプラットフォーム分野でcotomiを拡張\n::: more text";
        let expected = "Some text  more text";
        let result = filter_content(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_remove_target_blank_attributes() {
        let content = "Check this [link](https://example.com){target=\"_blank\"} for more info";
        let expected = "Check this [link](https://example.com) for more info";
        let result = filter_content(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_remove_multiple_patterns() {
        let content = r#"
Some content here.

:::small
画像の出典：GPT-4oによりLedge.aiが生成
:::

More content with a [link](https://example.com){target="_blank"} here.

:::box
関連記事：NECとさくらインターネットが協業、生成AIプラットフォーム分野でcotomiを拡張
:::

Final content.
        "#;
        let expected = r#"
Some content here.



More content with a [link](https://example.com) here.



Final content.
        "#;
        let result = filter_content(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_preserve_normal_content() {
        let content = "This is normal content with no special patterns.";
        let expected = "This is normal content with no special patterns.";
        let result = filter_content(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_debug_actual_patterns() {
        // Test the actual pattern we see in RSS output
        let content = r#"[発表](https://example.com){target="_blank"}した"#;
        let expected = r#"[発表](https://example.com)した"#;
        let result = filter_content(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_remove_html_encoded_target_blank() {
        // Test HTML-encoded {target="_blank"} pattern
        let content = r#"<a href="https://example.com">link</a>{target=&quot;_blank&quot;} text"#;
        let expected = r#"<a href="https://example.com">link</a> text"#;
        let result = filter_content(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_remove_double_encoded_target_blank() {
        // Test double HTML-encoded {target="_blank"} pattern as seen in actual RSS
        let content =
            r#"<a href="https://example.com">発表</a>{target=&amp;quot;_blank&amp;quot;}した"#;
        let expected = r#"<a href="https://example.com">発表</a>した"#;
        let result = filter_content(content);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_remove_all_target_blank_variants() {
        // Test all three variants of target="_blank" in one content
        let content = r#"Link1{target="_blank"} Link2{target=&quot;_blank&quot;} Link3{target=&amp;quot;_blank&amp;quot;} end"#;
        let expected = r#"Link1 Link2 Link3 end"#;
        let result = filter_content(content);
        assert_eq!(result, expected);
    }
}

use regex::Regex;

/// Content filter for cleaning up RSS article content
/// Removes unwanted patterns like image attributions, related article boxes, and link attributes
#[allow(dead_code)]
pub fn filter_content(content: &str) -> String {
    let mut result = content.to_string();

    // Remove :::small ... ::: blocks (image attributions)
    let small_pattern = Regex::new(r":::small[\s\S]*?:::").unwrap();
    result = small_pattern.replace_all(&result, "").to_string();

    // Remove :::box ... ::: blocks (related articles)
    let box_pattern = Regex::new(r":::box[\s\S]*?:::").unwrap();
    result = box_pattern.replace_all(&result, "").to_string();

    // Remove {target="_blank"} attributes
    let target_pattern = Regex::new(r#"\{target="_blank"\}"#).unwrap();
    result = target_pattern.replace_all(&result, "").to_string();

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
}

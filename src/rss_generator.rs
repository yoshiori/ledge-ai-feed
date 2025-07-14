use crate::rss_item::RssItem;
use quick_xml::events::{BytesDecl, Event};
use quick_xml::Writer;
use rss::{ChannelBuilder, ItemBuilder};
use std::io::Cursor;

pub fn generate_rss(items: Vec<RssItem>) -> Result<String, Box<dyn std::error::Error>> {
    let mut channel = ChannelBuilder::default()
        .title("Ledge.ai 新着記事")
        .link("https://ledge.ai/")
        .description("Ledge.ai の最新テクノロジー記事")
        .build();

    let rss_items: Vec<rss::Item> = items
        .into_iter()
        .map(|item| {
            ItemBuilder::default()
                .title(Some(item.title))
                .link(Some(item.link))
                .description(Some(item.description))
                .pub_date(Some(item.pub_date))
                .build()
        })
        .collect();

    channel.set_items(rss_items);

    // Get unformatted XML string from rss crate
    let xml_string = channel.to_string();

    // Format the XML with indentation
    format_xml(&xml_string)
}

fn format_xml(xml: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut reader = quick_xml::Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    // Copy events from reader to writer with formatting
    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(Event::Decl(_)) => {
                // Replace the original declaration with UTF-8
                writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
            }
            Ok(event) => writer.write_event(event)?,
            Err(e) => return Err(Box::new(e)),
        }
    }

    let result = writer.into_inner().into_inner();
    Ok(String::from_utf8(result)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rss_item::RssItem;

    #[test]
    fn test_generate_rss_creates_valid_xml() {
        let items = vec![
            RssItem {
                title: "Article 1".to_string(),
                link: "https://example.com/1".to_string(),
                description: "<p>Content 1</p>".to_string(),
                pub_date: "2025-01-14T10:00:00+09:00".to_string(),
            },
            RssItem {
                title: "Article 2".to_string(),
                link: "https://example.com/2".to_string(),
                description: "<p>Content 2</p>".to_string(),
                pub_date: "2025-01-14T11:00:00+09:00".to_string(),
            },
        ];

        let result = generate_rss(items);
        assert!(result.is_ok());

        let rss_content = result.unwrap();
        assert!(rss_content.contains("<title>Ledge.ai 新着記事</title>"));
        assert!(rss_content.contains("<link>https://ledge.ai/</link>"));
        assert!(rss_content.contains("<description>Ledge.ai の最新テクノロジー記事</description>"));
        assert!(rss_content.contains("<title>Article 1</title>"));
        assert!(rss_content.contains("<title>Article 2</title>"));

        // Check that XML is properly formatted with indentation
        assert!(rss_content.contains("\n"));
        assert!(rss_content.contains("  "));
    }

    #[test]
    fn test_xml_formatting() {
        let xml = r#"<?xml version="1.0"?><root><child>text</child></root>"#;
        let formatted = format_xml(xml);
        assert!(formatted.is_ok());

        let result = formatted.unwrap();
        assert!(result.contains("\n"));
        assert!(result.contains("  <child>"));
    }
}

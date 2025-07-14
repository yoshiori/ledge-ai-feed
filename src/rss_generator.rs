use crate::rss_item::RssItem;
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

    // Use rss crate's built-in pretty_write_to method for formatted XML output
    let mut buffer = Cursor::new(Vec::new());
    channel.pretty_write_to(&mut buffer, b' ', 2)?;

    let pretty_xml = String::from_utf8(buffer.into_inner())?;
    Ok(pretty_xml)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rss_item::RssItem;
    use std::io::Cursor;

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
    fn test_rss_crate_pretty_write_to() {
        let items = vec![RssItem {
            title: "Test Article".to_string(),
            link: "https://example.com/test".to_string(),
            description: "<p>Test Content</p>".to_string(),
            pub_date: "2025-01-14T10:00:00+09:00".to_string(),
        }];

        let mut channel = rss::ChannelBuilder::default()
            .title("Test RSS")
            .link("https://example.com/")
            .description("Test RSS Feed")
            .build();

        let rss_items: Vec<rss::Item> = items
            .into_iter()
            .map(|item| {
                rss::ItemBuilder::default()
                    .title(Some(item.title))
                    .link(Some(item.link))
                    .description(Some(item.description))
                    .pub_date(Some(item.pub_date))
                    .build()
            })
            .collect();

        channel.set_items(rss_items);

        // Test pretty_write_to method from rss crate
        let mut buffer = Cursor::new(Vec::new());
        let result = channel.pretty_write_to(&mut buffer, b' ', 2);

        assert!(result.is_ok());

        let pretty_xml = String::from_utf8(buffer.into_inner()).unwrap();
        println!("Pretty XML output:\n{}", pretty_xml);

        // Check that XML is properly formatted with indentation
        assert!(pretty_xml.contains("\n"));
        assert!(pretty_xml.contains("  "));
        assert!(pretty_xml.contains("<title>Test RSS</title>"));
    }

    #[test]
    fn test_rss_pretty_formatting() {
        let items = vec![RssItem {
            title: "Simple Article".to_string(),
            link: "https://example.com/simple".to_string(),
            description: "Simple Content".to_string(),
            pub_date: "2025-01-14T12:00:00+09:00".to_string(),
        }];

        let result = generate_rss(items);
        assert!(result.is_ok());

        let rss_content = result.unwrap();
        // Check that XML is properly formatted with indentation using rss crate's pretty_write_to
        assert!(rss_content.contains("\n"));
        assert!(rss_content.contains("  <channel>"));
        assert!(rss_content.contains("    <title>"));
    }
}

use crate::rss_item::RssItem;
use rss::{ChannelBuilder, ItemBuilder};

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
    Ok(channel.to_string())
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
    }
}
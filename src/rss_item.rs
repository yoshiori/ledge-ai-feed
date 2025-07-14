pub struct RssItem {
    pub title: String,
    pub link: String,
    pub description: String,
    pub pub_date: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rss_item_has_required_fields() {
        let item = RssItem {
            title: "Test Title".to_string(),
            link: "https://example.com".to_string(),
            description: "Test Description".to_string(),
            pub_date: "2025-01-14".to_string(),
        };

        assert_eq!(item.title, "Test Title");
        assert_eq!(item.link, "https://example.com");
        assert_eq!(item.description, "Test Description");
        assert_eq!(item.pub_date, "2025-01-14");
    }
}
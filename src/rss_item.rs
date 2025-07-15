use chrono::{DateTime, Utc};

pub struct RssItem {
    pub title: String,
    pub link: String,
    pub description: String,
    pub pub_date: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rss_item_has_required_fields() {
        let test_date = DateTime::parse_from_rfc3339("2025-01-14T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let item = RssItem {
            title: "Test Title".to_string(),
            link: "https://example.com".to_string(),
            description: "Test Description".to_string(),
            pub_date: test_date,
        };

        assert_eq!(item.title, "Test Title");
        assert_eq!(item.link, "https://example.com");
        assert_eq!(item.description, "Test Description");
        assert_eq!(item.pub_date, test_date);
    }
}

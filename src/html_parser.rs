use scraper::{Html, Selector};

#[derive(Debug, PartialEq)]
pub struct ArticleInfo {
    pub title: String,
    pub url: String,
    pub date: String,
}

pub fn parse_articles_from_html(
    html: &str,
) -> Result<Vec<ArticleInfo>, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html);
    let article_selector = Selector::parse("div.article-item")?;
    let link_selector = Selector::parse("a")?;
    let title_selector = Selector::parse("h3.article-title")?;
    let date_selector = Selector::parse("span.article-date")?;

    let mut articles = Vec::new();

    for element in document.select(&article_selector) {
        if let Some(link_element) = element.select(&link_selector).next() {
            if let Some(href) = link_element.value().attr("href") {
                if let Some(title_element) = element.select(&title_selector).next() {
                    if let Some(date_element) = element.select(&date_selector).next() {
                        let title = title_element.text().collect::<String>().trim().to_string();
                        let date = date_element.text().collect::<String>().trim().to_string();
                        let url = if href.starts_with('/') {
                            format!("https://ledge.ai{}", href)
                        } else {
                            href.to_string()
                        };

                        articles.push(ArticleInfo { title, url, date });
                    }
                }
            }
        }
    }

    Ok(articles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_articles_from_html() {
        let html = r#"
            <html>
                <body>
                    <div class="article-item">
                        <a href="/articles/test1">
                            <h3 class="article-title">Test Article 1</h3>
                            <span class="article-date">2025/1/14 [TUE]</span>
                        </a>
                    </div>
                    <div class="article-item">
                        <a href="/articles/test2">
                            <h3 class="article-title">Test Article 2</h3>
                            <span class="article-date">2025/1/13 [MON]</span>
                        </a>
                    </div>
                </body>
            </html>
        "#;

        let articles = parse_articles_from_html(html).unwrap();
        assert_eq!(articles.len(), 2);

        assert_eq!(articles[0].title, "Test Article 1");
        assert_eq!(articles[0].url, "https://ledge.ai/articles/test1");
        assert_eq!(articles[0].date, "2025/1/14 [TUE]");

        assert_eq!(articles[1].title, "Test Article 2");
        assert_eq!(articles[1].url, "https://ledge.ai/articles/test2");
        assert_eq!(articles[1].date, "2025/1/13 [MON]");
    }
}

use std::time::Duration;
use reqwest::Client;

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("ledge-ai-feed/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub async fn fetch_url(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let response = self.client.get(url).send().await?;
        let text = response.text().await?;
        Ok(text)
    }

    pub async fn fetch_url_with_timeout(
        &self,
        url: &str,
        timeout_ms: u64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .user_agent("ledge-ai-feed/1.0")
            .build()?;

        let response = client.get(url).send().await?;
        let text = response.text().await?;
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_html() {
        let client = HttpClient::new();
        
        // This is a minimal test - in practice we would use a mock HTTP client
        let _html = r#"<html><body><h1>Test</h1></body></html>"#;
        
        // For now, we'll test the timeout and error handling functionality
        let result = client.fetch_url_with_timeout("https://httpbin.org/delay/10", 1000).await;
        assert!(result.is_err()); // Should timeout
    }

    #[test]
    fn test_http_client_creation() {
        let _client = HttpClient::new();
        // Just test that we can create a client without panicking
        assert!(true);
    }
}
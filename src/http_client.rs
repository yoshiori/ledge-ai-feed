use reqwest::Client;
use std::time::Duration;

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_html() {
        let client = HttpClient::new();

        // This is a minimal test - in practice we would use a mock HTTP client
        // Test that client can be created and has the expected structure
        let _result = client.fetch_url("https://example.com").await;
        // In a real test, we would mock the HTTP response
    }

    #[test]
    fn test_http_client_creation() {
        let _client = HttpClient::new();
        // Just test that we can create a client without panicking
    }
}

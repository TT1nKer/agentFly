pub struct RelayClient {
    pub url: String,
}

impl RelayClient {
    pub fn new(url: &str) -> Self {
        RelayClient { url: url.to_string() }
    }

    pub async fn connect(&self) -> Result<(), String> {
        Ok(())
    }
}

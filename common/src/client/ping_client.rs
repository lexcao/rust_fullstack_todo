use std::ops::Deref;
use anyhow::bail;
use crate::client::ScopeClient;

pub struct PingClient(ScopeClient);

impl From<ScopeClient> for PingClient {
    fn from(c: ScopeClient) -> Self {
        Self(c)
    }
}

impl Deref for PingClient {
    type Target = ScopeClient;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PingClient {
    pub async fn ping(&self) -> anyhow::Result<String> {
        let request = self.inner.get(&format!("{}/ping", self.endpoint));

        let response = request.send().await?;

        if response.status() != 200 {
            bail!("server is unavailable <{}>", response.status());
        }

        let data = response.text().await?;

        Ok(data)
    }

    pub async fn health(&self) -> bool {
        self.ping().await
            .map(|pong| pong == "pong".to_string())
            .unwrap_or_else(|_| false)
    }
}

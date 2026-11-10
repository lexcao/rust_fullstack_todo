use common::client::ScopeClient;

use crate::helper::spawn_server;

mod helper;

#[tokio::test]
async fn test_ping_pong() -> anyhow::Result<()> {
    let endpoint = spawn_server();

    let client = ScopeClient::default()
        .endpoint(&endpoint)
        .ping_client();

    let pong = client.ping().await?;

    assert_eq!(pong, "pong".to_string());
    assert!(client.health().await);

    Ok(())
}

#[tokio::test]
async fn test_ping_server_unavailable() -> anyhow::Result<()> {
    let client = ScopeClient::default()
        .endpoint("http://127.0.0.1:9999")
        .ping_client();

    let error = client.ping().await
        .map_err(|e| e.downcast::<reqwest::Error>().unwrap())
        .err()
        .unwrap();

    assert!(error.is_connect());
    // reqwest 0.13 的 Display 不再带底层原因，沿 source 链找
    let chain = std::iter::successors(Some(&error as &dyn std::error::Error), |e| e.source())
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(": ");
    assert!(chain.to_lowercase().contains("connection refused"), "{chain}");
    assert!(!client.health().await);

    Ok(())
}

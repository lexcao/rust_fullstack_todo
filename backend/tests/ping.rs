use common::client::ScopeClient;

use common::client::ScopeClient;

use crate::helper::spawn_server;

mod helper;

#[actix_web::test]
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

#[actix_web::test]
async fn test_ping_server_unavailable() -> anyhow::Result<()> {
    let client = ScopeClient::default()
        .endpoint("http://127.0.0.1:9999")
        .ping_client();

    let error = client.ping().await
        .map_err(|e| e.downcast::<reqwest::Error>().unwrap())
        .err()
        .unwrap();

    assert!(error.is_connect());
    assert!(error.to_string().contains("Connection refused"));
    assert!(!client.health().await);

    Ok(())
}

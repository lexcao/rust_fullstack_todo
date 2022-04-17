mod todo;

use reqwest::Client;
pub use todo::*;

use reqwest::header::HeaderMap;

pub const NS: &str = "testing/integration";

#[derive(Clone)]
pub struct ScopeClient {
    endpoint: String,
    namespace: Option<String>,
    pub inner: reqwest::Client,
}

impl Default for ScopeClient {
    fn default() -> Self {
        Self {
            endpoint: "".to_string(),
            namespace: None,
            inner: Client::default(),
        }
    }
}

impl ScopeClient {
    pub fn endpoint(self, endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            ..self
        }
    }

    pub fn namespace(self, namespace: &str) -> Self {
        Self {
            namespace: Some(namespace.to_string()),
            ..self
        }
    }

    pub fn todo_client(&self) -> TodoClient {
        let mut c = self.clone();
        c.inner = client(self.namespace.clone());
        TodoClient::from(c)
    }
}

pub(crate) fn client(namespace: Option<String>) -> reqwest::Client {
    let mut default_headers = HeaderMap::new();
    if namespace.is_some() {
        let namespace = namespace.unwrap().parse().unwrap();
        default_headers.insert("t-ns", namespace);
    }

    reqwest::Client::builder()
        .default_headers(default_headers)
        .build()
        .expect("Failed to create reqwest Client")
}

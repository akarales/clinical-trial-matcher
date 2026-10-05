//! Shared state.

use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct AppState {
    pub http: reqwest::Client,
}

static HTTP: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

impl Default for AppState {
    fn default() -> Self {
        Self { http: HTTP.clone() }
    }
}

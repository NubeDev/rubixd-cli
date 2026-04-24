//! HTTP client — the only module that knows the base URL and issues requests.
//!
//! Every API call returns `anyhow::Result<serde_json::Value>` so that
//! `output.rs` can decide whether to pretty-print structured JSON or render
//! a human-readable summary. On a non-2xx response the error message contains
//! the HTTP status plus any `"error"` field from the JSON body.

use anyhow::{bail, Context, Result};
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::{Body, StatusCode};
use serde_json::Value;

/// Thin wrapper around a [`reqwest::Client`] bound to one base URL.
pub struct Client {
    inner: reqwest::Client,
    base: String,
}

impl Client {
    /// Build a client. `token` is optional; endpoints that require auth will
    /// return an error at request time if it is absent.
    pub fn new(base_url: &str, token: Option<&str>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        if let Some(tok) = token {
            let mut val = HeaderValue::from_str(&format!("Bearer {tok}"))
                .context("invalid token — contains non-ASCII bytes")?;
            val.set_sensitive(true);
            headers.insert(header::AUTHORIZATION, val);
        }
        let inner = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("failed to build HTTP client")?;
        Ok(Self {
            inner,
            base: base_url.trim_end_matches('/').to_string(),
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    /// `GET <path>` — deserialise response as JSON.
    pub async fn get(&self, path: &str) -> Result<Value> {
        let resp = self
            .inner
            .get(self.url(path))
            .send()
            .await
            .with_context(|| format!("GET {path}"))?;
        self.json_or_err(resp, path).await
    }

    /// `GET <path>?<query>` — deserialise response as JSON.
    pub async fn get_query(&self, path: &str, query: &[(&str, String)]) -> Result<Value> {
        let resp = self
            .inner
            .get(self.url(path))
            .query(query)
            .send()
            .await
            .with_context(|| format!("GET {path}"))?;
        self.json_or_err(resp, path).await
    }

    /// `POST <path>` with a JSON body — deserialise response as JSON.
    pub async fn post_json(&self, path: &str, body: &Value) -> Result<Value> {
        let resp = self
            .inner
            .post(self.url(path))
            .json(body)
            .send()
            .await
            .with_context(|| format!("POST {path}"))?;
        self.json_or_err(resp, path).await
    }

    /// `POST <path>` with a JSON body and no request body — empty-body POST.
    pub async fn post_empty(&self, path: &str) -> Result<Value> {
        let resp = self
            .inner
            .post(self.url(path))
            .header(header::CONTENT_LENGTH, "0")
            .send()
            .await
            .with_context(|| format!("POST {path}"))?;
        self.json_or_err(resp, path).await
    }

    /// `POST <path>` with raw bytes as the body — used for bundle push.
    pub async fn post_bytes(&self, path: &str, bytes: Vec<u8>) -> Result<Value> {
        let resp = self
            .inner
            .post(self.url(path))
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .body(Body::from(bytes))
            .send()
            .await
            .with_context(|| format!("POST {path}"))?;
        self.json_or_err(resp, path).await
    }

    /// Extract a JSON body from a successful response, or surface the API
    /// error message for a non-2xx status.
    async fn json_or_err(&self, resp: reqwest::Response, path: &str) -> Result<Value> {
        let status = resp.status();
        if status.is_success() {
            resp.json::<Value>()
                .await
                .with_context(|| format!("deserialising response from {path}"))
        } else {
            // Try to extract the `"error"` field blockd always includes.
            let api_msg: Option<String> = resp
                .json::<Value>()
                .await
                .ok()
                .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from));
            let detail = api_msg.unwrap_or_else(|| status.to_string());
            bail!("HTTP {}: {}", status_short(status), detail);
        }
    }
}

fn status_short(s: StatusCode) -> String {
    format!("{} {}", s.as_u16(), s.canonical_reason().unwrap_or(""))
}

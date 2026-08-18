//! The `/agent/v1` client.
//!
//! Alabasta's agent surface is a small JSON-RPC-over-HTTP API: every method is a
//! `POST` to `{site}/agent/v1/<method>` with a bearer token. Upstream's
//! `@alabasta/mcp` npm package is a ~30-line shim over exactly these endpoints,
//! so calling them directly loses nothing and avoids making a Node process a
//! hard dependency of starting a session.
//!
//! Every call blocks. Run it on `cx.background_executor()`.

use std::sync::{Arc, Mutex};

use anyhow::{Context, anyhow, bail};
use serde_json::{Value, json};

use super::AlabastaConnection;
use super::auth;
use crate::http;

/// Holds the credential and keeps the access token fresh.
///
/// This is the only place an Alabasta token exists in memory, and it never
/// leaves: agents reach Alabasta through the local bridge, which talks back to
/// this client rather than carrying a token of its own.
pub struct AlabastaClient {
    connection: AlabastaConnection,
    tokens: Mutex<Option<ActiveTokens>>,
}

struct ActiveTokens {
    access: String,
    refresh: String,
    expires_at: u64,
}

impl AlabastaClient {
    pub fn new(connection: AlabastaConnection) -> Arc<Self> {
        Arc::new(Self {
            connection,
            tokens: Mutex::new(None),
        })
    }

    pub fn connection(&self) -> &AlabastaConnection {
        &self.connection
    }

    /// Calls one `/agent/v1` method.
    ///
    /// A 401 is retried exactly once after refreshing, which covers the ordinary
    /// case of an access token that expired mid-session. A second 401 is a real
    /// authentication failure and is surfaced rather than looped on.
    pub fn call(&self, method: &str, body: &Value) -> anyhow::Result<Value> {
        match self.call_once(method, body, false) {
            Err(error) if is_unauthorized(&error) => self.call_once(method, body, true),
            result => result,
        }
    }

    fn call_once(&self, method: &str, body: &Value, force_refresh: bool) -> anyhow::Result<Value> {
        let token = self.access_token(force_refresh)?;
        let url = format!(
            "{}/agent/v1/{method}",
            self.connection.site_url.trim_end_matches('/')
        );
        let mut headers = vec![
            format!("Authorization: Bearer {token}"),
            "Content-Type: application/json".to_owned(),
        ];
        if !self.connection.workspace_id.trim().is_empty() {
            headers.push(format!("X-Alabasta-Workspace: {}", self.connection.workspace_id.trim()));
        }
        let response = http::request(
            "POST",
            &url,
            &headers,
            Some(&body.to_string()),
        )?;
        if response.status == 401 {
            bail!("unauthorized");
        }
        if !response.is_success() {
            let described = serde_json::from_str::<Value>(&response.body)
                .ok()
                .and_then(|value| value.get("error")?.as_str().map(str::to_owned))
                .unwrap_or_else(|| format!("HTTP {}", response.status));
            bail!("Alabasta could not answer {method}: {described}");
        }
        serde_json::from_str(&response.body)
            .with_context(|| format!("Alabasta returned unreadable JSON for {method}"))
    }

    fn access_token(&self, force_refresh: bool) -> anyhow::Result<String> {
        let mut guard = self
            .tokens
            .lock()
            .map_err(|_| anyhow!("token lock poisoned"))?;
        if !force_refresh {
            if let Some(tokens) = guard.as_ref() {
                if !auth::is_expired(tokens.expires_at) {
                    return Ok(tokens.access.clone());
                }
            }
        }
        let account = self.connection.account_key();
        let refresh_token = guard
            .as_ref()
            .map(|tokens| tokens.refresh.clone())
            .or_else(|| auth::load_refresh_token(&account))
            .ok_or_else(|| anyhow!("Anastasia is not signed in to Alabasta"))?;
        let issued = auth::refresh(&self.connection.site_url, &refresh_token)?;
        // Alabasta rotates refresh tokens: the old one is revoked on use, so the
        // new one has to be persisted immediately or the next refresh fails.
        auth::store_refresh_token(&account, &issued.refresh_token)?;
        let access = issued.access_token.clone();
        *guard = Some(ActiveTokens {
            access: access.clone(),
            refresh: issued.refresh_token,
            expires_at: auth::expiry_from(issued.expires_in),
        });
        Ok(access)
    }

    // ── The methods Anastasia actually uses ─────────────────────────────────

    /// The compiled context package for a task — Alabasta's L1 layer.
    pub fn task_context(&self, task_id: &str) -> anyhow::Result<Value> {
        self.call("task.context", &json!({ "taskId": task_id, "identifier": task_id }))
    }

    /// Standing rules — Alabasta's L0 layer, the stable prefix for a workspace.
    pub fn standing_context(&self, product_id: Option<&str>) -> anyhow::Result<Value> {
        let mut body = json!({});
        if let Some(product_id) = product_id {
            body["productId"] = json!(product_id);
        }
        self.call("standing.get", &body)
    }

    /// Tasks assigned to the signed-in user, for the session task picker.
    pub fn my_tasks(&self) -> anyhow::Result<Value> {
        self.call("tasks.listMine", &json!({}))
    }

    pub fn task(&self, identifier: &str) -> anyhow::Result<Value> {
        self.call("task.get", &json!({ "identifier": identifier }))
    }

    /// L3: read one `alabasta://` resource by URI.
    pub fn read_resource(&self, uri: &str) -> anyhow::Result<Value> {
        self.call("resource.read", &json!({ "uri": uri }))
    }

    /// L4: ranked keyword search returning pointers.
    pub fn search(&self, query: &str, limit: Option<u32>) -> anyhow::Result<Value> {
        let mut body = json!({ "query": query });
        if let Some(limit) = limit {
            body["limit"] = json!(limit);
        }
        self.call("context.search", &body)
    }

    /// Moves a task to in-progress. The one write V1 performs.
    pub fn start_work(&self, task_id: &str) -> anyhow::Result<Value> {
        self.call("task.startWork", &json!({ "taskId": task_id }))
    }
}

fn is_unauthorized(error: &anyhow::Error) -> bool {
    error.to_string() == "unauthorized"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_connection_needs_both_a_site_and_a_workspace() {
        let mut connection = AlabastaConnection::default();
        assert!(!connection.is_configured());
        connection.site_url = "https://x.convex.site".into();
        assert!(!connection.is_configured(), "a site alone is not a binding");
        connection.workspace_id = "ws1".into();
        assert!(connection.is_configured());
    }

    #[test]
    fn keychain_accounts_separate_deployments_and_identities() {
        // Two accounts on one deployment, or one account across a local and a
        // hosted deployment, must not overwrite each other's refresh token.
        let of = |site: &str, label: &str| {
            AlabastaConnection {
                site_url: site.into(),
                account_label: label.into(),
                ..AlabastaConnection::default()
            }
            .account_key()
        };
        assert_ne!(
            of("https://a.convex.site", "s@x"),
            of("https://b.convex.site", "s@x")
        );
        assert_ne!(
            of("https://a.convex.site", "s@x"),
            of("https://a.convex.site", "t@x")
        );
        assert_eq!(
            of("https://a.convex.site", "s@x"),
            of("https://a.convex.site", "s@x")
        );
    }

    #[test]
    fn only_a_401_triggers_a_refresh_retry() {
        assert!(is_unauthorized(&anyhow!("unauthorized")));
        // A 500 or a parse failure must not be mistaken for an expired token,
        // or a broken deployment turns into a token-refresh loop.
        assert!(!is_unauthorized(&anyhow!(
            "Alabasta could not answer task.context: HTTP 500"
        )));
        assert!(!is_unauthorized(&anyhow!("unreadable JSON")));
    }
}

//! Alabasta: shared workspace context for the agents Anastasia runs.
//!
//! Alabasta is the context and coordination plane; Anastasia is the local
//! execution plane. This module is the whole boundary between them, and it is
//! deliberately thin: Anastasia asks for a compiled context package and renders
//! it. It never decides which decisions are authoritative, never ranks context
//! by its own judgement, and never reimplements the Context Runtime. That lives
//! in Alabasta so its MCP transport, its web UI and Anastasia cannot drift apart
//! in what they consider in scope.
//!
//! Everything here is optional. With no connection the module resolves to
//! nothing and an agent launches exactly as it did before any of this existed.

pub mod auth;
pub mod client;
pub mod context;

pub use anastasia_protocol::alabasta::{AlabastaBinding, AlabastaConnection};

use anastasia_protocol::alabasta::{AlabastaLaunchRequest, AlabastaSessionBinding};
use anastasia_protocol::model::ProviderKind;

use crate::harness::HarnessContribution;

/// The Alabasta half of a launch: what to tell the agent, and what to record.
#[derive(Default)]
pub struct AlabastaResolution {
    pub contribution: HarnessContribution,
    pub binding: Option<AlabastaSessionBinding>,
}

/// Compiles a session's workspace context, or resolves to nothing.
///
/// Never returns an error and never blocks a launch: an unreachable deployment,
/// an expired credential or a task that has since been deleted all produce an
/// unhealthy binding and an empty contribution, and the agent still starts. The
/// status says what happened rather than implying the agent is informed.
pub fn resolve(
    provider: ProviderKind,
    request: Option<&AlabastaLaunchRequest>,
) -> AlabastaResolution {
    let Some(request) = request.filter(|request| request.connection.is_configured()) else {
        return AlabastaResolution::default();
    };
    let client = client::AlabastaClient::new(request.connection.clone());
    let compiled = context::compile(
        &client,
        &context::ContextRequest {
            provider,
            task_id: request.task_id.clone(),
            task_identifier: request.task_identifier.clone(),
            product_id: request.product_id.clone(),
        },
    );
    AlabastaResolution {
        contribution: compiled.contribution,
        binding: Some(AlabastaSessionBinding {
            task_id: request.task_id.clone(),
            task_identifier: request.task_identifier.clone(),
            task_title: request.task_title.clone(),
            status: compiled.status,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unbound_session_contributes_nothing() {
        for provider in ProviderKind::ALL {
            let resolution = resolve(provider, None);
            assert!(resolution.contribution.is_empty());
            assert!(resolution.binding.is_none());
        }
    }

    #[test]
    fn a_half_configured_connection_is_treated_as_signed_out() {
        // A settings file carrying a site but no workspace must not send a
        // request that can only 401; it means the user never finished connecting.
        let request = AlabastaLaunchRequest {
            connection: AlabastaConnection {
                site_url: "https://x.convex.site".into(),
                ..AlabastaConnection::default()
            },
            task_id: "t1".into(),
            task_identifier: "ALB-1".into(),
            task_title: "T".into(),
            product_id: None,
        };
        let resolution = resolve(ProviderKind::Claude, Some(&request));
        assert!(resolution.contribution.is_empty());
        assert!(resolution.binding.is_none());
    }

    #[test]
    fn an_unreachable_deployment_still_yields_a_launchable_session() {
        // Port 1 on loopback refuses immediately, standing in for "no network".
        let request = AlabastaLaunchRequest {
            connection: AlabastaConnection {
                site_url: "http://127.0.0.1:1".into(),
                workspace_id: "ws1".into(),
                account_label: "probe".into(),
                ..AlabastaConnection::default()
            },
            task_id: "t1".into(),
            task_identifier: "ALB-1".into(),
            task_title: "T".into(),
            product_id: None,
        };
        let resolution = resolve(ProviderKind::Claude, Some(&request));
        let binding = resolution.binding.expect("a bound session still reports");
        assert!(
            !binding.status.healthy,
            "an unreachable deployment claimed success"
        );
        assert!(binding.status.message.is_some());
        assert!(binding.status.sources.is_empty());
        // The launch itself is untouched, so the agent starts regardless.
        assert!(resolution.contribution.is_empty());
    }
}

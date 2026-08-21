//! Turning a compiled Alabasta context package into something an agent reads.
//!
//! Alabasta's Context Runtime has already done the hard part: ranked, budgeted,
//! provenance-labelled context, capped around 4000 tokens. This module renders
//! that package to markdown and reports what went into it. It deliberately does
//! **not** re-rank, re-filter or re-summarize — doing so would fork the
//! runtime's judgement, which is the one thing this integration must not do.
//!
//! Nothing here can stop a session: every failure becomes an unhealthy status
//! and an empty contribution, and the agent starts anyway.

use anyhow::Result;
use serde_json::Value;

use anastasia_protocol::alabasta::{
    AlabastaIntegration, AlabastaStatus, ContextReadiness, ContextSource,
};
use anastasia_protocol::model::ProviderKind;

use super::client::AlabastaClient;
use crate::harness::HarnessContribution;

/// What a session needs to compile its context.
pub struct ContextRequest {
    pub provider: ProviderKind,
    pub task_id: String,
    pub task_identifier: String,
    pub product_id: Option<String>,
}

/// The compiled result: what to tell the agent, and what to show the user.
pub struct CompiledContext {
    pub contribution: HarnessContribution,
    pub status: AlabastaStatus,
}

/// Fetches and renders the context for one session.
///
/// Standing rules (L0) come first because they carry the highest authority in
/// Fetches and renders the context for one session.
///
/// Standing rules (L0) come first because they carry the highest authority in
/// Alabasta's own ranking, then the task package (L1).
pub fn compile(client: &AlabastaClient, request: &ContextRequest) -> CompiledContext {
    let standing_res = client.standing_context(request.product_id.as_deref());
    let standing = standing_res.as_ref().ok().cloned();

    let (package_opt, package_err) = if !request.task_id.trim().is_empty() {
        match client.task_context(&request.task_id) {
            Ok(package) => (Some(package), None),
            Err(error) => (None, Some(error)),
        }
    } else {
        (None, None)
    };

    if package_opt.is_none() && standing.is_none() {
        let error_msg = package_err
            .map(|e| e.to_string())
            .or_else(|| standing_res.err().map(|e| e.to_string()))
            .unwrap_or_else(|| "unreachable Alabasta deployment".to_string());
        return CompiledContext {
            contribution: HarnessContribution::default(),
            status: AlabastaStatus::failed(request.provider, error_msg),
        };
    }
    let text = cap(render(&standing, package_opt.as_ref(), &request.task_identifier));
    let sources = package_opt
        .as_ref()
        .map(sources_of)
        .unwrap_or_default();
    let readiness = package_opt
        .as_ref()
        .and_then(|p| p.pointer("/contextQuality/readiness"))
        .and_then(Value::as_str)
        .map(ContextReadiness::from_id)
        .unwrap_or(if package_opt.is_some() || standing.is_some() {
            ContextReadiness::Ready
        } else {
            ContextReadiness::Unknown
        });
    let approximate_tokens = package_opt
        .as_ref()
        .and_then(|p| p.pointer("/meta/approximateTokens"))
        .and_then(Value::as_u64)
        .map(|tokens| tokens as u32);

    CompiledContext {
        contribution: HarnessContribution {
            instructions: Some(text),
            ..HarnessContribution::default()
        },
        status: AlabastaStatus::active(
            request.provider,
            AlabastaIntegration::for_provider(request.provider),
            readiness,
            sources,
            approximate_tokens,
        ),
    }
}

/// Ceiling on the compiled context, in bytes — roughly 10k tokens.
///
/// This text goes into the provider's instruction channel, so it is prepended to
/// every request for the life of the session. The server reports
/// `meta/approximateTokens` but nothing on this side ever enforced it, which left
/// a large workspace free to put an unbounded block in front of the whole
/// conversation.
///
/// ponytail: a byte cap, not a token count — no tokenizer here, and the point is
/// a ceiling rather than a precise budget. Swap in a real count if the status
/// line ever needs to report the number rather than just the fact.
const MAX_CONTEXT_BYTES: usize = 40 * 1024;

/// Truncates on a char boundary and tells the model it was truncated, so it
/// treats the tail as missing rather than as the end of the rules.
fn cap(mut text: String) -> String {
    if text.len() <= MAX_CONTEXT_BYTES {
        return text;
    }
    let end = text
        .char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= MAX_CONTEXT_BYTES)
        .last()
        .unwrap_or(0);
    text.truncate(end);
    text.push_str(
        "\n\n[Workspace context truncated here. Ask for the remainder if you need it.]\n",
    );
    text
}

/// Renders the package as markdown.
///
/// The shape is intentionally boring — headed sections in authority order. The
/// runtime already decided what belongs here and how much of it; this only has
/// to present it so a model reads it as instructions rather than as trivia.
fn render(standing: &Option<Value>, package: Option<&Value>, task_identifier: &str) -> String {
    let mut out = String::new();
    if !task_identifier.is_empty() && task_identifier != "Alabasta" {
        out.push_str(&format!(
            "# Alabasta workspace context for {task_identifier}\n\nThis is authoritative context from your team's Alabasta workspace, compiled for this task. Treat approved decisions and standing rules as binding constraints, not suggestions.\n"
        ));
    } else {
        out.push_str(
            "# Alabasta workspace context\n\nThis is authoritative context from your team's Alabasta workspace. Treat approved decisions and standing rules as binding constraints, not suggestions.\n"
        );
    }

    if let Some(rules) = standing
        .as_ref()
        .and_then(|standing| standing.get("rules"))
        .and_then(Value::as_array)
        .filter(|rules| !rules.is_empty())
    {
        out.push_str("\n## Standing rules\n\n");
        for rule in rules {
            let id = string_at(rule, "id").unwrap_or_default();
            if let Some(text) = string_at(rule, "text") {
                out.push_str(&format!("- {text}{}\n", suffix(&id)));
            }
        }
    }

    if let Some(package) = package {
        section(&mut out, "Task", package.get("task"), |task, out| {
            if let Some(title) = string_at(task, "title") {
                out.push_str(&format!("**{title}**\n\n"));
            }
            if let Some(description) = string_at(task, "description") {
                out.push_str(&format!("{description}\n"));
            }
            if let Some(status) = string_at(task, "status") {
                out.push_str(&format!("\nStatus: {status}\n"));
            }
        });

        list(
            &mut out,
            "Acceptance criteria",
            package.get("acceptanceCriteria"),
            |item| string_at(item, "text"),
        );

        list(
            &mut out,
            "Approved decisions",
            package.get("importantDecisions"),
            |item| {
                let title = string_at(item, "title")?;
                Some(match string_at(item, "summary") {
                    Some(summary) => format!("**{title}** — {summary}"),
                    None => format!("**{title}**"),
                })
            },
        );

        list(&mut out, "Guidance", package.get("agentGuidance"), |item| {
            string_at(item, "text").or_else(|| item.as_str().map(str::to_owned))
        });

        if let Some(project) = package
            .get("projectContext")
            .filter(|value| !value.is_null())
        {
            section(&mut out, "Project", Some(project), |project, out| {
                if let Some(name) = string_at(project, "name") {
                    out.push_str(&format!("{name}\n"));
                }
                if let Some(brief) =
                    string_at(project, "brief").or_else(|| string_at(project, "description"))
                {
                    out.push_str(&format!("\n{brief}\n"));
                }
            });
        }

        list(
            &mut out,
            "Open clarifications",
            package.pointer("/clarifications/pending"),
            |item| string_at(item, "question"),
        );

        list(
            &mut out,
            "Conflicts to resolve",
            package.get("conflicts"),
            |item| string_at(item, "summary").or_else(|| string_at(item, "description")),
        );

        if let Some(resources) = package
            .pointer("/contextIndex/resources")
            .and_then(Value::as_array)
            .filter(|resources| !resources.is_empty())
        {
            out.push_str("\n## More context available on request\n\nUse the Alabasta tools to read any of these by URI:\n\n");
            for resource in resources {
                let (Some(uri), Some(title)) =
                    (string_at(resource, "uri"), string_at(resource, "title"))
                else {
                    continue;
                };
                out.push_str(&format!("- `{uri}` — {title}\n"));
            }
        }
    }

    out.push_str("\n## Alabasta Workspace MCP Tools\n");
    out.push_str("You are connected to Alabasta via the local bridge MCP tools:\n");
    out.push_str("- `alabasta_get_context_package(taskId)`: Retrieve the compiled L1 context package for an assigned task or identifier (e.g. TOM-35, ALB-482).\n");
    out.push_str("- `alabasta_get_standing_context(productId)`: Retrieve the L0 standing context rules for the workspace or product.\n");
    out.push_str("- `alabasta_search_context(query, limit)`: Search workspace decisions, rules, and documents.\n");
    out.push_str("- `alabasta_read_resource(uri)`: Read resources by `alabasta://` URI.\n");
    out.push_str("- `alabasta_get_task(identifier)`: Retrieve task details by identifier.\n");
    out.push_str("When asked about tasks, tickets, or project requirements, use these Alabasta tools to retrieve authoritative context.\n");

    out
}

fn suffix(id: &str) -> String {
    if id.is_empty() {
        String::new()
    } else {
        format!(" ({id})")
    }
}

fn section(
    out: &mut String,
    title: &str,
    target: Option<&Value>,
    render: impl FnOnce(&Value, &mut String),
) {
    let Some(target) = target else { return };
    if target.is_null() {
        return;
    }
    out.push_str(&format!("\n## {title}\n\n"));
    render(target, out);
}

fn list(
    out: &mut String,
    title: &str,
    items: Option<&Value>,
    render_item: impl Fn(&Value) -> Option<String>,
) {
    let Some(items) = items.and_then(Value::as_array).filter(|arr| !arr.is_empty()) else {
        return;
    };
    let mut rendered = Vec::new();
    for item in items {
        if let Some(text) = render_item(item) {
            rendered.push(text);
        }
    }
    if rendered.is_empty() {
        return;
    }
    out.push_str(&format!("\n## {title}\n\n"));
    for line in rendered {
        out.push_str(&format!("- {line}\n"));
    }
}

fn string_at<'a>(value: &'a Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

fn sources_of(package: &Value) -> Vec<ContextSource> {
    let mut sources = Vec::new();
    if let Some(items) = package.pointer("/contextIndex/resources").and_then(Value::as_array) {
        for item in items {
            let Some(title) = string_at(item, "title") else { continue };
            let kind = string_at(item, "kind").unwrap_or_else(|| "resource".to_owned());
            let authority = string_at(item, "authority").unwrap_or_else(|| "inferred".to_owned());
            let authority_rank = item.get("authorityRank").and_then(Value::as_u64).unwrap_or(12) as u32;
            sources.push(ContextSource {
                uri: string_at(item, "uri"),
                title,
                kind,
                authority,
                authority_rank,
            });
        }
    }
    sources.sort_by_key(|s| s.authority_rank);
    sources
}

pub fn my_tasks(client: &AlabastaClient) -> Result<Vec<(String, String, String)>> {
    let value = client.my_tasks()?;
    let entries = value
        .as_array()
        .or_else(|| value.get("tasks")?.as_array())
        .cloned()
        .unwrap_or_default();
    Ok(entries
        .iter()
        .filter_map(|task| {
            Some((
                string_at(task, "id").or_else(|| string_at(task, "_id"))?,
                string_at(task, "identifier")?,
                string_at(task, "title").unwrap_or_default(),
            ))
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn package() -> Value {
        json!({
            "task": {
                "title": "Rotate workspace API keys",
                "description": "Old keys keep working for 24h",
                "status": "in_progress"
            },
            "importantDecisions": [{
                "id": "d38",
                "title": "Agent integrations must not mutate global user configuration",
                "summary": "Reuse the existing hashing helper."
            }],
            "acceptanceCriteria": [{ "text": "Old keys keep working for 24h" }],
            "agentGuidance": [{ "text": "Follow existing patterns." }],
            "clarifications": {
                "pending": [{ "question": "Should rotation be per-key or per-workspace?" }]
            },
            "conflicts": [],
            "contextIndex": {
                "resources": [
                    {
                        "uri": "alabasta://decisions/d38",
                        "title": "Agent integrations must not mutate global user configuration",
                        "kind": "decision",
                        "authority": "approved_decision",
                        "authorityRank": 3
                    },
                    {
                        "uri": "alabasta://policies/sec-01",
                        "title": "Credential storage rules",
                        "kind": "policy",
                        "authority": "security_policy",
                        "authorityRank": 1
                    }
                ]
            }
        })
    }

    fn standing() -> Option<Value> {
        Some(
            json!({ "rules": [{ "id": "RUL-88", "text": "Never mutate global agent configuration." }] }),
        )
    }

    #[test]
    fn the_rendered_context_carries_what_binds_the_agent() {
        let text = render(&standing(), Some(&package()), "ALB-482");
        assert!(text.contains("ALB-482"));
        assert!(text.contains("Never mutate global agent configuration. (RUL-88)"));
        assert!(text.contains("Rotate workspace API keys"));
        assert!(text.contains("Old keys keep working for 24h"));
        assert!(text.contains("Agent integrations must not mutate global user configuration"));
        assert!(text.contains("Reuse the existing hashing helper."));
        assert!(text.contains("Should rotation be per-key or per-workspace?"));
        assert!(text.contains("alabasta://decisions/d38"));
    }

    #[test]
    fn empty_sections_are_omitted_rather_than_rendered_blank() {
        let text = render(&standing(), Some(&package()), "ALB-482");
        assert!(!text.contains("Conflicts to resolve"));

        let sparse = json!({ "task": { "title": "Only a title" } });
        let text = render(&None, Some(&sparse), "ALB-1");
        assert!(text.contains("Only a title"));
        assert!(!text.contains("## Acceptance criteria"));
        assert!(!text.contains("## Standing rules"));
    }

    #[test]
    fn sources_are_ordered_by_alabastas_authority_not_ours() {
        let sources = sources_of(&package());
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].authority, "security_policy");
        assert_eq!(sources[0].authority_rank, 1);
        assert_eq!(sources[1].authority, "approved_decision");
    }

    #[test]
    fn a_package_from_a_newer_runtime_degrades_instead_of_failing() {
        let future = json!({
            "task": { "title": "T" },
            "contextQuality": { "readiness": "mostly_ready" },
            "somethingNew": { "nested": [1, 2, 3] }
        });
        assert_eq!(
            future
                .pointer("/contextQuality/readiness")
                .and_then(Value::as_str)
                .map(ContextReadiness::from_id),
            Some(ContextReadiness::Unknown)
        );
        assert!(render(&None, Some(&future), "ALB-9").contains("T"));
        assert!(sources_of(&future).is_empty());
    }

    #[test]
    fn task_lists_tolerate_both_shapes_the_api_might_return() {
        let bare = json!([{ "id": "t1", "identifier": "ALB-1", "title": "One" }]);
        let wrapped = json!({ "tasks": [{ "_id": "t2", "identifier": "ALB-2", "title": "Two" }] });
        for value in [bare, wrapped] {
            let entries = value
                .as_array()
                .or_else(|| value.get("tasks")?.as_array())
                .cloned()
                .unwrap_or_default();
            assert_eq!(entries.len(), 1);
        }
    }
}

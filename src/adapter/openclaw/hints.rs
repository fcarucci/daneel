// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::models::graph::{AgentEdge, AgentEdgeKind};

#[derive(Debug, Deserialize)]
struct OpenClawHintsConfig {
    #[serde(default)]
    agents: AgentsConfig,
    #[serde(default)]
    daneel: DaneelHintsConfig,
}

#[derive(Debug, Default, Deserialize)]
struct DaneelHintsConfig {
    #[serde(default)]
    relationship_hints: RelationshipHintsConfig,
}

#[derive(Debug, Deserialize)]
struct RelationshipHintsConfig {
    #[serde(default = "default_relationship_hints_enabled")]
    enabled: bool,
}

#[derive(Debug, Default, Deserialize)]
struct AgentsConfig {
    #[serde(default)]
    list: Vec<AgentConfigEntry>,
}

#[derive(Debug, Default, Deserialize)]
struct AgentConfigEntry {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default, rename = "agentDir")]
    agent_dir: Option<PathBuf>,
    #[serde(default)]
    subagents: SubagentsConfig,
}

#[derive(Debug, Default, Deserialize)]
struct SubagentsConfig {
    #[serde(default, rename = "allowAgents")]
    allow_agents: Vec<String>,
}

fn default_relationship_hints_enabled() -> bool {
    true
}

impl Default for RelationshipHintsConfig {
    fn default() -> Self {
        Self {
            enabled: default_relationship_hints_enabled(),
        }
    }
}

pub(super) fn load_agent_relationship_hints() -> Result<Vec<AgentEdge>, String> {
    let config_path = openclaw_config_path()?;
    load_agent_relationship_hints_from_path(&config_path)
}

pub(super) fn load_agent_relationship_hints_from_path(
    config_path: &Path,
) -> Result<Vec<AgentEdge>, String> {
    let raw = fs::read_to_string(config_path)
        .map_err(|error| format!("Could not read {}: {error}", config_path.display()))?;
    let parsed: OpenClawHintsConfig = serde_json::from_str(&raw)
        .map_err(|error| format!("Could not parse {}: {error}", config_path.display()))?;

    if !parsed.daneel.relationship_hints.enabled {
        return Ok(Vec::new());
    }

    let agent_root = config_path
        .parent()
        .map(|parent| parent.join("agents"))
        .unwrap_or_else(|| PathBuf::from(".").join("agents"));
    let alias_map = build_alias_map(&parsed.agents.list);
    let known_ids: BTreeSet<_> = parsed
        .agents
        .list
        .iter()
        .map(|agent| agent.id.clone())
        .collect();
    let mut edge_keys = BTreeSet::new();

    for agent in &parsed.agents.list {
        collect_config_hint_edges(agent, &known_ids, &mut edge_keys);

        let agent_dir = agent
            .agent_dir
            .clone()
            .unwrap_or_else(|| agent_root.join(&agent.id).join("agent"));

        collect_markdown_hint_edges(agent, &agent_dir, &alias_map, &mut edge_keys);
        collect_agent_json_hint_edges(agent, &agent_dir, &alias_map, &mut edge_keys);
    }

    Ok(edge_keys
        .into_iter()
        .map(|(source_id, target_id)| metadata_edge(&source_id, &target_id))
        .collect())
}

fn openclaw_config_path() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("OPENCLAW_CONFIG_PATH") {
        return Ok(PathBuf::from(path));
    }

    let home = env::var("HOME").map_err(|_| "HOME is not set.".to_string())?;
    Ok(PathBuf::from(home).join(".openclaw").join("openclaw.json"))
}

fn build_alias_map(agents: &[AgentConfigEntry]) -> BTreeMap<String, String> {
    let mut aliases = BTreeMap::new();

    for agent in agents {
        aliases.insert(normalize_agent_key(&agent.id), agent.id.clone());
        if !agent.name.trim().is_empty() {
            aliases.insert(normalize_agent_key(&agent.name), agent.id.clone());
        }
    }

    aliases
}

fn collect_config_hint_edges(
    agent: &AgentConfigEntry,
    known_ids: &BTreeSet<String>,
    edge_keys: &mut BTreeSet<(String, String)>,
) {
    for target_id in &agent.subagents.allow_agents {
        if known_ids.contains(target_id) && target_id != &agent.id {
            edge_keys.insert((agent.id.clone(), target_id.clone()));
        }
    }
}

fn collect_markdown_hint_edges(
    agent: &AgentConfigEntry,
    agent_dir: &Path,
    alias_map: &BTreeMap<String, String>,
    edge_keys: &mut BTreeSet<(String, String)>,
) {
    let agents_md_path = agent_dir.join("AGENTS.md");
    let Ok(markdown) = fs::read_to_string(&agents_md_path) else {
        return;
    };

    for target_id in parse_works_with_targets(&markdown, alias_map) {
        if target_id != agent.id {
            edge_keys.insert((agent.id.clone(), target_id));
        }
    }
}

fn collect_agent_json_hint_edges(
    agent: &AgentConfigEntry,
    agent_dir: &Path,
    alias_map: &BTreeMap<String, String>,
    edge_keys: &mut BTreeSet<(String, String)>,
) {
    let agent_json_path = agent_dir.join("agent.json");
    let Ok(raw) = fs::read_to_string(&agent_json_path) else {
        return;
    };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return;
    };

    for target_id in extract_delegate_targets(&parsed, alias_map) {
        if target_id != agent.id {
            edge_keys.insert((agent.id.clone(), target_id));
        }
    }
}

fn parse_works_with_targets(
    markdown: &str,
    alias_map: &BTreeMap<String, String>,
) -> BTreeSet<String> {
    let mut in_works_with = false;
    let mut targets = BTreeSet::new();

    for line in markdown.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            let heading = trimmed.trim_start_matches('#').trim();
            let normalized = normalize_agent_key(heading);
            if normalized == "works-with" {
                in_works_with = true;
                continue;
            }

            if in_works_with {
                break;
            }
        }

        if !in_works_with || !(trimmed.starts_with("- ") || trimmed.starts_with("* ")) {
            continue;
        }

        if let Some(target) = extract_markdown_target(trimmed, alias_map) {
            targets.insert(target);
        }
    }

    targets
}

fn extract_markdown_target(
    bullet_line: &str,
    alias_map: &BTreeMap<String, String>,
) -> Option<String> {
    let content = bullet_line
        .trim_start_matches("- ")
        .trim_start_matches("* ")
        .trim();

    if let Some(rest) = content.strip_prefix("**") {
        let end = rest.find("**")?;
        let candidate = &rest[..end];
        return resolve_agent_reference(candidate, alias_map);
    }

    let candidate = content.split(':').next().unwrap_or(content);
    resolve_agent_reference(candidate, alias_map)
}

fn extract_delegate_targets(
    value: &serde_json::Value,
    alias_map: &BTreeMap<String, String>,
) -> BTreeSet<String> {
    let mut targets = BTreeSet::new();
    collect_delegate_targets(value, alias_map, &mut targets);
    targets
}

fn collect_delegate_targets(
    value: &serde_json::Value,
    alias_map: &BTreeMap<String, String>,
    targets: &mut BTreeSet<String>,
) {
    match value {
        serde_json::Value::String(text) => {
            for candidate in delegation_candidates(text) {
                if let Some(target) = resolve_agent_reference(&candidate, alias_map) {
                    targets.insert(target);
                }
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                collect_delegate_targets(value, alias_map, targets);
            }
        }
        serde_json::Value::Object(map) => {
            for value in map.values() {
                collect_delegate_targets(value, alias_map, targets);
            }
        }
        _ => {}
    }
}

fn delegation_candidates(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let patterns = ["delegate to the ", "delegate to "];
    let mut candidates = Vec::new();

    for pattern in patterns {
        let mut search_start = 0;
        while let Some(index) = lower[search_start..].find(pattern) {
            let start = search_start + index + pattern.len();
            let remaining = &lower[start..];
            if let Some(end) = remaining.find(" agent") {
                let candidate = remaining[..end].trim();
                if !candidate.is_empty() {
                    candidates.push(candidate.to_string());
                }
            }
            search_start = start;
        }
    }

    candidates
}

fn resolve_agent_reference(
    candidate: &str,
    alias_map: &BTreeMap<String, String>,
) -> Option<String> {
    alias_map.get(&normalize_agent_key(candidate)).cloned()
}

fn normalize_agent_key(raw: &str) -> String {
    let mut normalized = raw.to_lowercase();
    normalized = normalized.replace("**", "");
    normalized = normalized
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == ' ' || ch == '-' || ch == '_' {
                ch
            } else {
                ' '
            }
        })
        .collect();

    let trimmed = normalized.trim().trim_end_matches(" agent").trim();
    trimmed
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .replace('_', "-")
}

fn metadata_edge(source_id: &str, target_id: &str) -> AgentEdge {
    AgentEdge {
        source_id: source_id.to_string(),
        target_id: target_id.to_string(),
        kind: AgentEdgeKind::MetadataHint,
    }
}

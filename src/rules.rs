use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub level: Level,
    pub kind: RuleKind,
    #[serde(default)]
    pub phrases: Vec<String>,
    #[serde(default)]
    pub first_phrases: Vec<String>,
    #[serde(default)]
    pub second_phrases: Vec<String>,
    #[serde(default)]
    pub threshold_per_1000_words: Option<f32>,
    #[serde(default)]
    pub min_occurrences: Option<usize>,
    pub message: String,
    #[serde(default)]
    pub suggestion: Option<String>,
    #[serde(default = "default_confidence")]
    pub confidence: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Ruleset {
    pub name: String,
    pub rules: Vec<Rule>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Info,
    Warning,
    Error,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        };
        f.write_str(value)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleKind {
    #[serde(rename = "phrase_presence")]
    Presence,
    #[serde(rename = "phrase_density")]
    Density,
    #[serde(rename = "phrase_pair_density")]
    PairDensity,
}

impl Default for Ruleset {
    fn default() -> Self {
        Self {
            name: "unnamed".into(),
            rules: Vec::new(),
        }
    }
}

fn default_confidence() -> f32 {
    1.0
}

pub fn load_builtin_rulesets(names: &[String]) -> Result<Vec<Rule>> {
    let mut rules = Vec::new();
    for name in names {
        let raw = match name.as_str() {
            "default" => include_str!("../rulesets/default.toml"),
            "zh" => include_str!("../rulesets/zh.toml"),
            unknown => bail!("unknown built-in ruleset: {unknown}"),
        };
        rules.extend(parse_ruleset(raw, name)?.rules);
    }
    Ok(merge_rules_by_id(rules))
}

pub fn load_ruleset_file(path: &Path) -> Result<Vec<Rule>> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read ruleset {}", path.display()))?;
    Ok(parse_ruleset(&raw, &path.display().to_string())?.rules)
}

#[cfg(test)]
pub fn builtin_rules() -> Result<Vec<Rule>> {
    load_builtin_rulesets(&["default".into(), "zh".into()])
}

pub fn merge_rules_by_id(rules: Vec<Rule>) -> Vec<Rule> {
    let mut merged: Vec<Rule> = Vec::new();
    for rule in rules {
        if let Some(existing) = merged.iter_mut().find(|existing| existing.id == rule.id) {
            *existing = rule;
        } else {
            merged.push(rule);
        }
    }
    merged
}

fn parse_ruleset(raw: &str, label: &str) -> Result<Ruleset> {
    toml::from_str(raw).with_context(|| format!("failed to parse ruleset {label}"))
}

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::rules::{Rule, load_builtin_rulesets, load_ruleset_file, merge_rules_by_id};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub sensitivity: Sensitivity,
    pub mode: Mode,
    pub uncertain_action: DiagnosticAction,
    pub file_extensions: Vec<String>,
    pub extends: Vec<String>,
    pub ruleset_paths: Vec<PathBuf>,
    pub rules: Vec<Rule>,
    pub mute: MuteConfig,
    pub custom_rules: Vec<Rule>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Sensitivity {
    Low,
    Normal,
    #[default]
    High,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Batch,
    #[default]
    Interactive,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticAction {
    Warn,
    #[default]
    AskUser,
    Off,
}

impl std::fmt::Display for DiagnosticAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Warn => "warn",
            Self::AskUser => "ask_user",
            Self::Off => "off",
        };
        f.write_str(value)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct MuteConfig {
    pub rules: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sensitivity: Sensitivity::High,
            mode: Mode::Interactive,
            uncertain_action: DiagnosticAction::AskUser,
            file_extensions: vec![
                "md".into(),
                "mdx".into(),
                "txt".into(),
                "rst".into(),
                "adoc".into(),
            ],
            extends: vec!["default".into(), "zh".into()],
            ruleset_paths: Vec::new(),
            rules: Vec::new(),
            mute: MuteConfig::default(),
            custom_rules: Vec::new(),
        }
    }
}

impl Config {
    pub fn load_hierarchy(cwd: PathBuf) -> Result<Self> {
        let mut config = Self::default();
        if let Some(user_config) = user_config_path().filter(|path| path.exists()) {
            config.merge_file(&user_config)?;
        }
        for path in project_config_paths(&cwd) {
            config.merge_file(&path)?;
        }
        Ok(config)
    }

    pub fn merge_file(&mut self, path: &Path) -> Result<()> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        let mut next: Config = toml::from_str(&raw)
            .with_context(|| format!("failed to parse config {}", path.display()))?;
        if let Some(parent) = path.parent() {
            for ruleset_path in &mut next.ruleset_paths {
                if ruleset_path.is_relative() {
                    *ruleset_path = parent.join(&ruleset_path);
                }
            }
        }
        self.merge(next);
        Ok(())
    }

    pub fn rules(&self) -> Result<Vec<Rule>> {
        let mut rules = load_builtin_rulesets(&self.extends)?;
        for path in &self.ruleset_paths {
            rules.extend(load_ruleset_file(path)?);
        }
        rules.extend(self.rules.clone());
        rules.extend(self.custom_rules.clone());
        Ok(merge_rules_by_id(rules)
            .into_iter()
            .filter(|rule| !self.mute.rules.iter().any(|muted| muted == &rule.id))
            .collect())
    }

    pub fn diagnostic_action(&self, confidence: f32) -> DiagnosticAction {
        if self.mode == Mode::Interactive
            && confidence < 0.8
            && self.uncertain_action != DiagnosticAction::Off
        {
            return self.uncertain_action;
        }
        DiagnosticAction::Warn
    }

    pub fn threshold_multiplier(&self) -> f32 {
        match self.sensitivity {
            Sensitivity::High => 0.75,
            Sensitivity::Normal => 1.0,
            Sensitivity::Low => 1.5,
        }
    }

    fn merge(&mut self, next: Config) {
        self.sensitivity = next.sensitivity;
        self.mode = next.mode;
        self.uncertain_action = next.uncertain_action;
        self.file_extensions = next.file_extensions;
        self.extends = next.extends;
        self.ruleset_paths.extend(next.ruleset_paths);
        self.rules.extend(next.rules);
        self.mute.rules.extend(next.mute.rules);
        self.mute.rules.sort();
        self.mute.rules.dedup();
        self.custom_rules.extend(next.custom_rules);
    }
}

fn user_config_path() -> Option<PathBuf> {
    let home = env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config/slop-lint/config.toml"))
}

fn project_config_paths(cwd: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for ancestor in cwd.ancestors() {
        for name in [".slop-lint.toml", "slop-lint.toml", "sloplint.toml"] {
            let candidate = ancestor.join(name);
            if candidate.exists() {
                paths.push(candidate);
            }
        }
    }
    paths.reverse();
    paths
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn loads_rules_from_relative_ruleset_path() {
        let dir = tempdir().unwrap();
        let ruleset_dir = dir.path().join("rulesets");
        fs::create_dir(&ruleset_dir).unwrap();
        fs::write(
            ruleset_dir.join("brand.toml"),
            r#"
name = "brand"

[[rules]]
id = "brand.empty"
name = "Brand empty phrase"
level = "warning"
kind = "phrase_presence"
phrases = ["world-class"]
message = "Avoid empty brand claims."
confidence = 0.9
"#,
        )
        .unwrap();
        let config_path = dir.path().join(".slop-lint.toml");
        fs::write(
            &config_path,
            r#"
extends = []
ruleset_paths = ["rulesets/brand.toml"]
"#,
        )
        .unwrap();

        let mut config = Config::default();
        config.merge_file(&config_path).unwrap();
        let rules = config.rules().unwrap();

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "brand.empty");
    }

    #[test]
    fn inline_rules_override_extended_rules() {
        let config: Config = toml::from_str(
            r#"
extends = ["default"]

[[rules]]
id = "slop.generic-conclusion"
name = "Overridden generic conclusion"
level = "error"
kind = "phrase_presence"
phrases = ["in conclusion"]
message = "Project-specific stricter message."
confidence = 1.0
"#,
        )
        .unwrap();

        let rules = config.rules().unwrap();
        let rule = rules
            .iter()
            .find(|rule| rule.id == "slop.generic-conclusion")
            .unwrap();

        assert_eq!(rule.level, crate::rules::Level::Error);
        assert_eq!(rule.message, "Project-specific stricter message.");
    }
}

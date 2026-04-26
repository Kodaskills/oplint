use crate::types::{Rule, Severity};
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefaultRulesConfig {
    pub rules: Vec<RuleConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    pub id: String,
    pub name: String,
    pub category: String,
    pub severity: Severity,
    pub message: String,
    pub suggestion: Option<String>,
    pub query: Option<String>,
    pub expect: Option<String>, // "match" (default) | "no-match"
    pub path_filter: Option<String>,
    pub except_in: Option<Vec<String>>,
    pub applies_to: Option<String>,
    pub options: Option<HashMap<String, serde_yaml::Value>>,
    pub accuracy: Option<String>,
    pub accuracy_note: Option<String>,
    pub skip_if_desktop_only: Option<bool>,
    pub reference: Option<String>,
    #[serde(rename = "use")]
    pub use_fn: Option<String>,
}

impl From<RuleConfig> for Rule {
    fn from(config: RuleConfig) -> Self {
        Rule {
            id: config.id,
            name: config.name,
            category: config.category,
            severity: config.severity,
            message: config.message,
            suggestion: config.suggestion,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleOverride {
    pub severity: Option<Severity>,
    pub disabled: Option<bool>,
    pub options: Option<HashMap<String, serde_yaml::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcludeConfig {
    #[serde(default = "default_true")]
    pub use_gitignore: bool,
    #[serde(default)]
    pub patterns: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl Default for ExcludeConfig {
    fn default() -> Self {
        Self {
            use_gitignore: true,
            patterns: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    pub extends: Option<String>,
    #[serde(rename = "rules", default)]
    pub rules_config: Option<UserRulesConfig>,
    #[serde(rename = "custom_rules", default)]
    pub custom_rules: Vec<RuleConfig>,
    #[serde(default)]
    pub exclude: ExcludeConfig,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRulesConfig {
    pub enabled: Option<RulesEnabled>,
    pub disabled: Option<Vec<String>>,
    pub skip_accuracy: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RulesEnabled {
    All(String),
    List(Vec<String>),
}

impl RulesEnabled {
    pub fn is_all(&self) -> bool {
        match self {
            RulesEnabled::All(s) => s.to_lowercase() == "all",
            RulesEnabled::List(_) => false,
        }
    }

    pub fn get_list(&self) -> Vec<String> {
        match self {
            RulesEnabled::All(s) if s.to_lowercase() == "all" => Vec::new(),
            RulesEnabled::All(_) => Vec::new(),
            RulesEnabled::List(list) => list.clone(),
        }
    }
}

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load(path: &Path) -> anyhow::Result<UserConfig> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let content = fs::read_to_string(path)?;
        match extension {
            "yaml" | "yml" => serde_yaml::from_str(&content).map_err(Into::into),
            "json" => serde_json::from_str(&content).map_err(Into::into),
            "toml" => toml::from_str(&content).map_err(Into::into),
            _ => Err(anyhow::anyhow!("Unsupported config format: {}", extension)),
        }
    }

    pub fn load_or_default(path: Option<&Path>) -> UserConfig {
        path.and_then(|p| Self::load(p).ok()).unwrap_or_default()
    }

    pub fn find_config(start: &Path) -> Option<PathBuf> {
        let config_names = [
            ".oplint.yaml",
            ".oplint.yml",
            ".oplint.json",
            ".oplint.toml",
            "oplint.yaml",
            "oplint.yml",
            "oplint.json",
            "oplint.toml",
        ];

        let mut current = Some(start.to_path_buf());
        while let Some(path) = current {
            for name in &config_names {
                let config_path = path.join(name);
                if config_path.exists() {
                    return Some(config_path);
                }
            }
            current = path.parent().map(|p| p.to_path_buf());
        }
        None
    }
}

pub struct RuleMatcher {
    enabled_rules: Vec<String>,
    disabled_rules: Vec<String>,
    rule_overrides: HashMap<String, RuleOverride>,
    skip_accuracy: Vec<String>,
}

impl RuleMatcher {
    pub fn new(config: &UserConfig) -> Self {
        let mut enabled_rules = Vec::new();
        let mut disabled_rules = Vec::new();
        let mut rule_overrides = HashMap::new();

        let mut skip_accuracy = Vec::new();

        if let Some(rules_config) = &config.rules_config {
            if let Some(enabled) = &rules_config.enabled {
                if enabled.is_all() {
                    enabled_rules = Vec::new();
                } else {
                    enabled_rules = enabled.get_list();
                }
            }
            if let Some(disabled) = &rules_config.disabled.clone() {
                disabled_rules.clone_from(disabled);
            }
            if let Some(sa) = &rules_config.skip_accuracy {
                skip_accuracy.clone_from(sa);
            }
        }

        for (id, value) in &config.extra {
            if let Ok(override_val) = serde_yaml::from_value::<RuleOverride>(value.clone()) {
                if override_val.disabled == Some(true) {
                    disabled_rules.push(id.clone());
                }
                rule_overrides.insert(id.clone(), override_val);
            }
        }

        Self {
            enabled_rules,
            disabled_rules,
            rule_overrides,
            skip_accuracy,
        }
    }

    pub fn is_accuracy_allowed(&self, accuracy: &str) -> bool {
        !self.skip_accuracy.iter().any(|a| a == accuracy)
    }

    /// True when the user explicitly disabled or whitelisted rules (other than via
    /// `skip_accuracy`). The score/grade is then based on a subset of guidelines.
    pub fn has_disabled_rules(&self) -> bool {
        !self.disabled_rules.is_empty() || !self.enabled_rules.is_empty()
    }

    pub fn is_enabled(&self, rule_id: &str) -> bool {
        if self.disabled_rules.contains(&rule_id.to_string()) {
            return false;
        }
        if self.enabled_rules.is_empty() {
            return true;
        }
        self.enabled_rules.contains(&rule_id.to_string())
    }

    pub fn get_override(&self, rule_id: &str) -> Option<&RuleOverride> {
        self.rule_overrides.get(rule_id)
    }

    pub fn apply_severity(&self, rule: &mut Rule) {
        if let Some(override_val) = self.get_override(&rule.id) {
            if let Some(severity) = &override_val.severity {
                rule.severity = severity.clone();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CustomRule {
    pub config: RuleConfig,
    pub path_filter: Option<Pattern>,
    pub except_patterns: Vec<Pattern>,
    pub applies_to: AppliesTo,
}

#[derive(Debug, Clone)]
pub enum AppliesTo {
    All,
    Manifest,
    Files,
    License,
}

impl CustomRule {
    pub fn from_config(config: RuleConfig) -> anyhow::Result<Self> {
        let path_filter = config
            .path_filter
            .as_ref()
            .map(|p| Pattern::new(p))
            .transpose()?;

        let except_patterns = config
            .except_in
            .as_ref()
            .map(|patterns| {
                patterns
                    .iter()
                    .filter_map(|p| Pattern::new(p).ok())
                    .collect()
            })
            .unwrap_or_default();

        let applies_to = match config.applies_to.as_deref() {
            Some("manifest") => AppliesTo::Manifest,
            Some("files") => AppliesTo::Files,
            Some("license") => AppliesTo::License,
            _ => AppliesTo::All,
        };

        Ok(Self {
            config,
            path_filter,
            except_patterns,
            applies_to,
        })
    }

    pub fn matches_file(&self, path: &Path) -> bool {
        if let Some(filter) = &self.path_filter {
            if !filter.matches(path.to_str().unwrap_or("")) {
                return false;
            }
        }
        for except in &self.except_patterns {
            if except.matches(path.to_str().unwrap_or("")) {
                return false;
            }
        }
        true
    }

    pub fn is_manifest_rule(&self) -> bool {
        matches!(self.applies_to, AppliesTo::Manifest)
    }

    pub fn is_license_rule(&self) -> bool {
        matches!(self.applies_to, AppliesTo::License)
    }
}

pub fn load_default_rules() -> Vec<RuleConfig> {
    let yaml = include_str!("../default_rules.yaml");
    let config: DefaultRulesConfig = serde_yaml::from_str(yaml).unwrap_or_default();
    config.rules
}

use oplint::config::{RuleMatcher, RuleOverride, RulesEnabled, UserConfig, UserRulesConfig};
use oplint::types::Severity;
use std::collections::HashMap;

#[test]
fn test_rule_matcher_all_enabled() {
    let config = UserConfig::default();
    let matcher = RuleMatcher::new(&config);

    assert!(matcher.is_enabled("SEC001"));
    assert!(matcher.is_enabled("ANY_ID"));
}

#[test]
fn test_rule_matcher_disabled_list() {
    let config = UserConfig {
        rules_config: Some(UserRulesConfig {
            enabled: None,
            disabled: Some(vec!["SEC001".to_string()]),
            skip_accuracy: None,
        }),
        ..Default::default()
    };

    let matcher = RuleMatcher::new(&config);
    assert!(!matcher.is_enabled("SEC001"));
    assert!(matcher.is_enabled("SEC002"));
}

#[test]
fn test_rule_matcher_enabled_list() {
    let config = UserConfig {
        rules_config: Some(UserRulesConfig {
            enabled: Some(RulesEnabled::List(vec!["SEC001".to_string()])),
            disabled: None,
            skip_accuracy: None,
        }),
        ..Default::default()
    };

    let matcher = RuleMatcher::new(&config);
    assert!(matcher.is_enabled("SEC001"));
    assert!(!matcher.is_enabled("SEC002"));
}

#[test]
fn test_rule_matcher_overrides() {
    let mut extra = HashMap::new();
    let ov = RuleOverride {
        severity: Some(Severity::Error),
        disabled: Some(true),
        options: None,
    };
    extra.insert("SEC001".to_string(), serde_yaml::to_value(ov).unwrap());

    let config = UserConfig {
        extra,
        ..Default::default()
    };

    let matcher = RuleMatcher::new(&config);
    assert!(!matcher.is_enabled("SEC001"));

    let ov_get = matcher.get_override("SEC001").unwrap();
    assert_eq!(ov_get.severity, Some(Severity::Error));
}

#[test]
fn test_accuracy_filtering() {
    let config = UserConfig {
        rules_config: Some(UserRulesConfig {
            enabled: None,
            disabled: None,
            skip_accuracy: Some(vec!["approximate".to_string()]),
        }),
        ..Default::default()
    };

    let matcher = RuleMatcher::new(&config);
    assert!(matcher.is_accuracy_allowed("exact"));
    assert!(!matcher.is_accuracy_allowed("approximate"));
}

#[test]
fn has_disabled_rules_false_when_all_enabled() {
    let config = UserConfig::default();
    let matcher = RuleMatcher::new(&config);
    assert!(!matcher.has_disabled_rules());
}

#[test]
fn has_disabled_rules_false_when_only_skip_accuracy() {
    // skip_accuracy alone must NOT trigger partial coverage
    let config = UserConfig {
        rules_config: Some(UserRulesConfig {
            enabled: None,
            disabled: None,
            skip_accuracy: Some(vec!["approximate".to_string()]),
        }),
        ..Default::default()
    };
    let matcher = RuleMatcher::new(&config);
    assert!(!matcher.has_disabled_rules());
}

#[test]
fn has_disabled_rules_true_when_disabled_list() {
    let config = UserConfig {
        rules_config: Some(UserRulesConfig {
            enabled: None,
            disabled: Some(vec!["SEC001".to_string()]),
            skip_accuracy: None,
        }),
        ..Default::default()
    };
    let matcher = RuleMatcher::new(&config);
    assert!(matcher.has_disabled_rules());
}

#[test]
fn has_disabled_rules_true_when_enabled_whitelist() {
    // Whitelist mode: only named rules run — others are excluded
    let config = UserConfig {
        rules_config: Some(UserRulesConfig {
            enabled: Some(RulesEnabled::List(vec!["SEC001".to_string()])),
            disabled: None,
            skip_accuracy: None,
        }),
        ..Default::default()
    };
    let matcher = RuleMatcher::new(&config);
    assert!(matcher.has_disabled_rules());
}

#[test]
fn has_disabled_rules_true_when_override_disabled() {
    let mut extra = HashMap::new();
    let ov = RuleOverride {
        severity: None,
        disabled: Some(true),
        options: None,
    };
    extra.insert("SEC001".to_string(), serde_yaml::to_value(ov).unwrap());
    let config = UserConfig {
        extra,
        ..Default::default()
    };
    let matcher = RuleMatcher::new(&config);
    assert!(matcher.has_disabled_rules());
}

use crate::config::{
    load_default_rules, AppliesTo, ConfigLoader, CustomRule, RuleMatcher, UserConfig,
};
use crate::types::{Severity, Violation};
use glob::Pattern;
use std::path::Path;
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator};

struct CompiledRule {
    query: Option<Query>,
    rule_id: String,
    rule_category: String,
    rule_message: String,
    rule_severity: Severity,
    rule_suggestion: Option<String>,
    expect_match: bool,
    path_filter: Option<Pattern>,
    except_patterns: Vec<Pattern>,
    accuracy: String,
    accuracy_note: Option<String>,
    skip_if_desktop_only: bool,
    use_fn: Option<String>,
    rule_reference: Option<String>,
}

pub struct Linter {
    file_rules: Vec<CompiledRule>,
    manifest_rules: Vec<CompiledRule>,
    license_rules: Vec<CompiledRule>,
    ts_language: Language,
    json_language: Language,
    is_desktop_only: bool,
    pub partial_coverage: bool,
    plugin_id: Option<String>,
    plugin_name: Option<String>,
}

fn init_languages() -> (Language, Language) {
    // SAFETY: Both constants are C externs returning stable `TSLanguage` pointers compatible
    // with this tree-sitter version. `from_raw` is the 0.22-era API; 0.23+ has `Language::new`.
    let ts = unsafe {
        Language::from_raw((tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into_raw())()
            as *const tree_sitter::ffi::TSLanguage)
    };
    // SAFETY: same as above, for the JSON grammar.
    let json = unsafe {
        Language::from_raw(
            (tree_sitter_json::LANGUAGE.into_raw())() as *const tree_sitter::ffi::TSLanguage
        )
    };
    (ts, json)
}

fn compile_query_for_rule(
    lang: &Language,
    rule_id: &str,
    query_src: Option<&str>,
    use_fn: Option<&str>,
) -> Option<Option<Query>> {
    if use_fn.is_some() {
        return Some(None);
    }
    match query_src {
        Some(q) if !q.trim().is_empty() => match Query::new(lang, q) {
            Ok(q) => Some(Some(q)),
            Err(e) => {
                eprintln!("[oplint] Failed to compile query for {rule_id}: {e}");
                None
            }
        },
        _ => None,
    }
}

fn compile_builtin_rule(
    rule_config: &crate::config::RuleConfig,
    lang: &Language,
    rule_matcher: &crate::config::RuleMatcher,
) -> Option<CompiledRule> {
    let query = compile_query_for_rule(
        lang,
        &rule_config.id,
        rule_config.query.as_deref(),
        rule_config.use_fn.as_deref(),
    )?;

    let mut severity = rule_config.severity.clone();
    if let Some(ov) = rule_matcher.get_override(&rule_config.id) {
        if let Some(s) = &ov.severity {
            severity = s.clone();
        }
    }

    Some(CompiledRule {
        query,
        rule_id: rule_config.id.clone(),
        rule_category: rule_config.category.clone(),
        rule_message: rule_config.message.clone(),
        rule_severity: severity,
        rule_suggestion: rule_config.suggestion.clone(),
        expect_match: rule_config.expect.as_deref() != Some("no-match"),
        path_filter: rule_config
            .path_filter
            .as_ref()
            .and_then(|p| Pattern::new(p).ok()),
        except_patterns: rule_config
            .except_in
            .as_ref()
            .map(|ps| ps.iter().filter_map(|p| Pattern::new(p).ok()).collect())
            .unwrap_or_default(),
        accuracy: rule_config
            .accuracy
            .clone()
            .unwrap_or_else(|| "approximate".to_string()),
        accuracy_note: rule_config.accuracy_note.clone(),
        skip_if_desktop_only: rule_config.skip_if_desktop_only.unwrap_or(false),
        use_fn: rule_config.use_fn.clone(),
        rule_reference: rule_config.reference.clone(),
    })
}

fn compile_custom_rule(cr: &crate::config::CustomRule, lang: &Language) -> Option<CompiledRule> {
    let query = compile_query_for_rule(
        lang,
        &cr.config.id,
        cr.config.query.as_deref(),
        cr.config.use_fn.as_deref(),
    )?;

    Some(CompiledRule {
        query,
        rule_id: cr.config.id.clone(),
        rule_category: cr.config.category.clone(),
        rule_message: cr.config.message.clone(),
        rule_severity: cr.config.severity.clone(),
        rule_suggestion: cr.config.suggestion.clone(),
        expect_match: cr.config.expect.as_deref() != Some("no-match"),
        path_filter: cr.path_filter.clone(),
        except_patterns: cr.except_patterns.clone(),
        accuracy: cr
            .config
            .accuracy
            .clone()
            .unwrap_or_else(|| "approximate".to_string()),
        accuracy_note: cr.config.accuracy_note.clone(),
        skip_if_desktop_only: cr.config.skip_if_desktop_only.unwrap_or(false),
        use_fn: cr.config.use_fn.clone(),
        rule_reference: cr.config.reference.clone(),
    })
}

fn bucket_rule(
    compiled: CompiledRule,
    is_manifest: bool,
    is_license: bool,
    file_rules: &mut Vec<CompiledRule>,
    manifest_rules: &mut Vec<CompiledRule>,
    license_rules: &mut Vec<CompiledRule>,
) {
    if is_manifest {
        manifest_rules.push(compiled);
    } else if is_license {
        license_rules.push(compiled);
    } else {
        file_rules.push(compiled);
    }
}

impl Linter {
    pub fn new_with_config(config_path: Option<&Path>) -> Self {
        let (ts_language, json_language) = init_languages();

        let default_rules = load_default_rules();

        let user_config = if let Some(path) = config_path {
            ConfigLoader::load(path).unwrap_or_default()
        } else {
            UserConfig::default()
        };

        let rule_matcher = RuleMatcher::new(&user_config);
        let partial_coverage = rule_matcher.has_disabled_rules();

        let custom_rules: Vec<CustomRule> = user_config
            .custom_rules
            .iter()
            .filter(|r| rule_matcher.is_enabled(&r.id))
            .filter_map(|r| CustomRule::from_config(r.clone()).ok())
            .collect();

        let mut file_rules: Vec<CompiledRule> = Vec::new();
        let mut manifest_rules: Vec<CompiledRule> = Vec::new();
        let mut license_rules: Vec<CompiledRule> = Vec::new();

        for rule_config in &default_rules {
            if !rule_matcher.is_enabled(&rule_config.id) {
                continue;
            }
            let effective_accuracy = rule_config.accuracy.as_deref().unwrap_or("approximate");
            if !rule_matcher.is_accuracy_allowed(effective_accuracy) {
                continue;
            }

            let is_manifest = rule_config.applies_to.as_deref() == Some("manifest");
            let is_license = rule_config.applies_to.as_deref() == Some("license");
            let lang = if is_manifest {
                &json_language
            } else {
                &ts_language
            };

            let Some(compiled) = compile_builtin_rule(rule_config, lang, &rule_matcher) else {
                continue;
            };

            bucket_rule(
                compiled,
                is_manifest,
                is_license,
                &mut file_rules,
                &mut manifest_rules,
                &mut license_rules,
            );
        }

        for cr in &custom_rules {
            let effective_accuracy = cr.config.accuracy.as_deref().unwrap_or("approximate");
            if !rule_matcher.is_accuracy_allowed(effective_accuracy) {
                continue;
            }

            let is_manifest = matches!(cr.applies_to, AppliesTo::Manifest);
            let is_license = matches!(cr.applies_to, AppliesTo::License);
            let lang = if is_manifest {
                &json_language
            } else {
                &ts_language
            };

            let Some(compiled) = compile_custom_rule(cr, lang) else {
                continue;
            };

            bucket_rule(
                compiled,
                is_manifest,
                is_license,
                &mut file_rules,
                &mut manifest_rules,
                &mut license_rules,
            );
        }

        Self {
            file_rules,
            manifest_rules,
            license_rules,
            ts_language,
            json_language,
            is_desktop_only: false,
            partial_coverage,
            plugin_id: None,
            plugin_name: None,
        }
    }

    pub fn new(_rules: Vec<crate::types::Rule>) -> Self {
        Self::new_with_config(None)
    }

    pub fn set_is_desktop_only(&mut self, value: bool) {
        self.is_desktop_only = value;
    }

    pub fn set_plugin_id(&mut self, id: &str) {
        self.plugin_id = Some(id.to_string());
    }

    pub fn set_plugin_name(&mut self, name: &str) {
        self.plugin_name = Some(name.to_string());
    }

    pub fn total_active_weight(&self) -> f64 {
        self.file_rules
            .iter()
            .chain(self.manifest_rules.iter())
            .chain(self.license_rules.iter())
            .map(|r| crate::types::severity_weight(&r.rule_severity))
            .sum::<f64>()
            .max(1.0)
    }

    pub fn all_categories(&self) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        let mut cats = Vec::new();
        for rule in self
            .file_rules
            .iter()
            .chain(self.manifest_rules.iter())
            .chain(self.license_rules.iter())
        {
            if seen.insert(rule.rule_category.clone()) {
                cats.push(rule.rule_category.clone());
            }
        }
        cats
    }

    pub fn detect_desktop_only(manifest_content: &str) -> bool {
        if let Some(idx) = manifest_content.find("\"isDesktopOnly\"") {
            let after = &manifest_content[idx + "\"isDesktopOnly\"".len()..];
            let val = after.trim_start().trim_start_matches(':').trim_start();
            return val.starts_with("true");
        }
        false
    }

    pub fn lint_file(&self, path: &Path, content: &str) -> Vec<Violation> {
        let mut parser = Parser::new();
        if parser.set_language(&self.ts_language).is_err() {
            return Vec::new();
        }

        let tree = match parser.parse(content, None) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let mut violations = Vec::new();
        let content_bytes = content.as_bytes();

        for rule in &self.file_rules {
            if rule.skip_if_desktop_only && self.is_desktop_only {
                continue;
            }

            if !rule_matches_path(rule, path) {
                continue;
            }

            if let Some(ref fn_name) = rule.use_fn {
                let ctx = crate::checks::CheckContext {
                    rule_id: &rule.rule_id,
                    rule_category: &rule.rule_category,
                    rule_message: &rule.rule_message,
                    rule_severity: rule.rule_severity.clone(),
                    rule_suggestion: rule.rule_suggestion.as_deref(),
                    accuracy: &rule.accuracy,
                    accuracy_note: rule.accuracy_note.as_deref(),
                    plugin_id: self.plugin_id.as_deref(),
                    plugin_name: self.plugin_name.as_deref(),
                };
                let mut vs = crate::checks::dispatch(fn_name, path, content, &ctx);
                let ref_str = rule.rule_reference.as_deref();
                for v in &mut vs {
                    v.reference = ref_str.map(|s| s.to_string());
                }
                violations.extend(vs);
                continue;
            }

            let query = match &rule.query {
                Some(q) => q,
                None => continue,
            };

            let mut cursor = QueryCursor::new();
            let mut has_match = false;
            // Collect (line, source) pairs inline — captures reference cursor-internal memory
            // and must be read before the cursor advances to the next match.
            let mut match_lines: Vec<(usize, Option<String>)> = Vec::new();

            let mut matches = cursor.matches(query, tree.root_node(), content_bytes);
            while let Some(m) = matches.next() {
                has_match = true;
                if rule.expect_match {
                    if let Some(capture) = m.captures.first() {
                        let line = capture.node.start_position().row + 1;
                        let src = content.lines().nth(line - 1).map(|l| l.trim().to_string());
                        match_lines.push((line, src));
                    }
                }
            }

            if rule.expect_match {
                for (line, src) in match_lines {
                    let mut v = Violation::new(
                        &rule.rule_id,
                        &rule.rule_category,
                        &rule.rule_message,
                        rule.rule_severity.clone(),
                        path.to_path_buf(),
                        line,
                    );
                    if let Some(src) = src {
                        v = v.with_source_code(&src);
                    }
                    if let Some(ref s) = rule.rule_suggestion {
                        v = v.with_suggestion(s);
                    }
                    v = v.with_accuracy(&rule.accuracy, rule.accuracy_note.as_deref());
                    v = v.with_reference(rule.rule_reference.as_deref());
                    violations.push(v);
                }
            } else if !has_match {
                let mut v = Violation::new(
                    &rule.rule_id,
                    &rule.rule_category,
                    &rule.rule_message,
                    rule.rule_severity.clone(),
                    path.to_path_buf(),
                    1,
                );
                if let Some(ref s) = rule.rule_suggestion {
                    v = v.with_suggestion(s);
                }
                v = v.with_accuracy(&rule.accuracy, rule.accuracy_note.as_deref());
                v = v.with_reference(rule.rule_reference.as_deref());
                violations.push(v);
            }
        }

        violations
    }

    pub fn lint_manifest(&self, path: &Path, content: &str) -> Vec<Violation> {
        let mut parser = Parser::new();
        if parser.set_language(&self.json_language).is_err() {
            return Vec::new();
        }

        let tree = match parser.parse(content, None) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let mut violations = Vec::new();
        let content_bytes = content.as_bytes();

        for rule in &self.manifest_rules {
            if !rule_matches_path(rule, path) {
                continue;
            }

            // Dispatch special Rust functions instead of tree-sitter
            if let Some(ref fn_name) = rule.use_fn {
                let ctx = crate::checks::CheckContext {
                    rule_id: &rule.rule_id,
                    rule_category: &rule.rule_category,
                    rule_message: &rule.rule_message,
                    rule_severity: rule.rule_severity.clone(),
                    rule_suggestion: rule.rule_suggestion.as_deref(),
                    accuracy: &rule.accuracy,
                    accuracy_note: rule.accuracy_note.as_deref(),
                    plugin_id: self.plugin_id.as_deref(),
                    plugin_name: self.plugin_name.as_deref(),
                };
                let mut vs = crate::checks::dispatch(fn_name, path, content, &ctx);
                let ref_str = rule.rule_reference.as_deref();
                for v in &mut vs {
                    v.reference = ref_str.map(|s| s.to_string());
                }
                violations.extend(vs);
                continue;
            }

            let query = match &rule.query {
                Some(q) => q,
                None => continue,
            };

            let mut cursor = QueryCursor::new();
            let mut has_match = false;
            let mut match_lines: Vec<usize> = Vec::new();

            let mut matches = cursor.matches(query, tree.root_node(), content_bytes);
            while let Some(m) = matches.next() {
                has_match = true;
                if rule.expect_match {
                    if let Some(capture) = m.captures.first() {
                        match_lines.push(capture.node.start_position().row + 1);
                    }
                }
            }

            if rule.expect_match {
                for line in match_lines {
                    let mut v = Violation::new(
                        &rule.rule_id,
                        &rule.rule_category,
                        &rule.rule_message,
                        rule.rule_severity.clone(),
                        path.to_path_buf(),
                        line,
                    );
                    if let Some(ref s) = rule.rule_suggestion {
                        v = v.with_suggestion(s);
                    }
                    v = v.with_accuracy(&rule.accuracy, rule.accuracy_note.as_deref());
                    v = v.with_reference(rule.rule_reference.as_deref());
                    violations.push(v);
                }
            } else if !has_match {
                let mut v = Violation::new(
                    &rule.rule_id,
                    &rule.rule_category,
                    &rule.rule_message,
                    rule.rule_severity.clone(),
                    path.to_path_buf(),
                    1,
                );
                if let Some(ref s) = rule.rule_suggestion {
                    v = v.with_suggestion(s);
                }
                v = v.with_accuracy(&rule.accuracy, rule.accuracy_note.as_deref());
                v = v.with_reference(rule.rule_reference.as_deref());
                violations.push(v);
            }
        }

        violations
    }

    pub fn lint_license(&self, path: &Path, content: &str) -> Vec<Violation> {
        let mut violations = Vec::new();
        for rule in &self.license_rules {
            if let Some(ref fn_name) = rule.use_fn {
                let ctx = crate::checks::CheckContext {
                    rule_id: &rule.rule_id,
                    rule_category: &rule.rule_category,
                    rule_message: &rule.rule_message,
                    rule_severity: rule.rule_severity.clone(),
                    rule_suggestion: rule.rule_suggestion.as_deref(),
                    accuracy: &rule.accuracy,
                    accuracy_note: rule.accuracy_note.as_deref(),
                    plugin_id: self.plugin_id.as_deref(),
                    plugin_name: self.plugin_name.as_deref(),
                };
                let mut vs = crate::checks::dispatch(fn_name, path, content, &ctx);
                let ref_str = rule.rule_reference.as_deref();
                for v in &mut vs {
                    v.reference = ref_str.map(|s| s.to_string());
                }
                violations.extend(vs);
            }
        }
        violations
    }
}

fn rule_matches_path(rule: &CompiledRule, path: &Path) -> bool {
    if let Some(filter) = &rule.path_filter {
        if !filter.matches(path.to_str().unwrap_or("")) {
            return false;
        }
    }
    for except in &rule.except_patterns {
        if except.matches(path.to_str().unwrap_or("")) {
            return false;
        }
    }
    true
}

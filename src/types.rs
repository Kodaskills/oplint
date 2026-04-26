use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    pub fn as_str(&self) -> &str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "error" => Some(Severity::Error),
            "warning" => Some(Severity::Warning),
            "info" => Some(Severity::Info),
            _ => None,
        }
    }
}

pub fn severity_weight(s: &Severity) -> f64 {
    match s {
        Severity::Error => 10.0,
        Severity::Warning => 5.0,
        Severity::Info => 1.0,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub rule_id: String,
    pub category: String,
    pub message: String,
    pub severity: Severity,
    pub file: PathBuf,
    pub line: usize,
    pub column: Option<usize>,
    pub suggestion: Option<String>,
    pub source_code: Option<String>,
    pub accuracy: Option<String>,
    pub accuracy_note: Option<String>,
    pub reference: Option<String>,
}

impl Violation {
    pub fn new(
        rule_id: &str,
        category: &str,
        message: &str,
        severity: Severity,
        file: PathBuf,
        line: usize,
    ) -> Self {
        Self {
            rule_id: rule_id.to_string(),
            category: category.to_string(),
            message: message.to_string(),
            severity,
            file,
            line,
            column: None,
            suggestion: None,
            source_code: None,
            accuracy: None,
            accuracy_note: None,
            reference: None,
        }
    }

    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }

    pub fn with_source_code(mut self, source: &str) -> Self {
        self.source_code = Some(source.to_string());
        self
    }

    pub fn with_accuracy(mut self, accuracy: &str, note: Option<&str>) -> Self {
        self.accuracy = Some(accuracy.to_string());
        self.accuracy_note = note.map(|s| s.to_string());
        self
    }

    pub fn with_reference(mut self, reference: Option<&str>) -> Self {
        self.reference = reference.map(|s| s.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub category: String,
    pub severity: Severity,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub rules: Vec<String>,
    pub severity_threshold: Option<Severity>,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub plugin_name: Option<String>,
    pub total_files: usize,
    pub total_violations: usize,
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
    pub score: u8,
    pub grade: String,
    pub grade_label: String,
    pub duration_ms: u64,
    pub avg_file_ms: u64,
    pub min_file_ms: u64,
    pub max_file_ms: u64,
    /// True when the user explicitly disabled or whitelisted specific rules (not just
    /// `skip_accuracy`). Score and grade cover only the active subset of guidelines.
    pub partial_coverage: bool,
    /// Ordered unique list of all rule categories that were checked (including those with 0 violations).
    pub all_categories: Vec<String>,
    /// Simple weighted sum of violations for display (error=10, warning=5, info=1).
    pub weighted_penalty: f64,
    /// Sum of severity weights across all active rules — the score denominator.
    pub total_active_weight: f64,
}

impl Default for Summary {
    fn default() -> Self {
        Self {
            plugin_name: None,
            total_files: 0,
            total_violations: 0,
            errors: 0,
            warnings: 0,
            infos: 0,
            score: 100,
            grade: "A+".to_string(),
            grade_label: "Perfect".to_string(),
            duration_ms: 0,
            avg_file_ms: 0,
            min_file_ms: 0,
            max_file_ms: 0,
            partial_coverage: false,
            all_categories: Vec::new(),
            weighted_penalty: 0.0,
            total_active_weight: 0.0,
        }
    }
}

impl Summary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_timing(&mut self, total_ms: u64, per_file: &[u64]) {
        self.duration_ms = total_ms;
        if !per_file.is_empty() {
            self.min_file_ms = *per_file.iter().min().unwrap();
            self.max_file_ms = *per_file.iter().max().unwrap();
            let sum: u64 = per_file.iter().sum();
            self.avg_file_ms = sum / per_file.len() as u64;
        }
    }

    pub fn set_rule_baseline(&mut self, total_active_weight: f64) {
        self.total_active_weight = total_active_weight;
    }

    pub fn add_violation(&mut self, v: &Violation) {
        self.total_violations += 1;
        match v.severity {
            Severity::Error => self.errors += 1,
            Severity::Warning => self.warnings += 1,
            Severity::Info => self.infos += 1,
        }
        self.weighted_penalty += severity_weight(&v.severity);
    }

    /// Compute score using SCORE_PROGRESS algorithm:
    /// - Per violated rule: penalty = weight × (1 + log2(max_occurrences_in_one_file))
    /// - Score = 100 × (1 − total_penalty / total_active_weight)
    pub fn finalize(&mut self, violations: &[Violation]) {
        if violations.is_empty() {
            self.score = 100;
            self.grade = "A+".to_string();
            self.grade_label = "Perfect".to_string();
            return;
        }

        // Count occurrences per (rule_id, file)
        let mut per_rule_file: HashMap<(String, String), usize> = HashMap::new();
        for v in violations {
            let file = v.file.to_str().unwrap_or("").to_string();
            *per_rule_file.entry((v.rule_id.clone(), file)).or_insert(0) += 1;
        }

        // Per rule: max occurrences in any single file + severity (from first seen violation)
        let mut rule_max: HashMap<String, (usize, Severity)> = HashMap::new();
        for v in violations {
            let file = v.file.to_str().unwrap_or("").to_string();
            let count = per_rule_file[&(v.rule_id.clone(), file)];
            let entry = rule_max
                .entry(v.rule_id.clone())
                .or_insert((0, v.severity.clone()));
            if count > entry.0 {
                entry.0 = count;
            }
        }

        // total_penalty = Σ weight × (1 + log2(max_occurrences))
        let total_penalty: f64 = rule_max
            .values()
            .map(|(max_occ, sev)| severity_weight(sev) * (1.0 + (*max_occ as f64).log2()))
            .sum();

        let budget = self.total_active_weight.max(1.0);
        let raw = ((1.0 - (total_penalty / budget).min(1.0)) * 100.0)
            .round()
            .clamp(0.0, 100.0);
        self.score = raw as u8;

        let (g, l) = match self.score {
            100 => ("A+", "Perfect"),
            90..=99 => ("A", "Excellent"),
            80..=89 => ("B", "Good"),
            70..=79 => ("C", "Fair"),
            60..=69 => ("D", "Poor"),
            _ => ("F", "Critical"),
        };
        self.grade = g.to_string();
        self.grade_label = l.to_string();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum OutputFormat {
    #[default]
    Terminal,
    Json,
    Sarif,
    Html,
    Checkstyle,
}

impl OutputFormat {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "sarif" => Some(Self::Sarif),
            "html" => Some(Self::Html),
            "checkstyle" => Some(Self::Checkstyle),
            _ => Some(Self::Terminal),
        }
    }
}

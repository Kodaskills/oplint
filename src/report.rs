#[cfg(any(feature = "fmt-html", feature = "fmt-markdown"))]
use askama::Template;
#[cfg(any(feature = "fmt-html", feature = "fmt-markdown"))]
use serde::Serialize;

#[cfg(feature = "fmt-html")]
#[derive(Template)]
#[template(path = "report.html")]
pub struct ReportTemplate<'a> {
    pub report_date: &'a str,
    pub summary: SummaryView,
    pub errors: Vec<ViolationGroupView<'a>>,
    pub warnings: Vec<ViolationGroupView<'a>>,
    pub infos: Vec<ViolationGroupView<'a>>,
}

#[cfg(feature = "fmt-markdown")]
#[derive(Template)]
#[template(path = "report.md")]
pub struct MdReportTemplate<'a> {
    pub report_date: &'a str,
    pub summary: SummaryView,
    pub errors: Vec<ViolationGroupView<'a>>,
    pub warnings: Vec<ViolationGroupView<'a>>,
    pub infos: Vec<ViolationGroupView<'a>>,
}

#[cfg(any(feature = "fmt-html", feature = "fmt-markdown"))]
#[derive(Debug, Clone, Serialize)]
pub struct SummaryView {
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
    pub partial_coverage: bool,
    pub all_categories_json: String,
    pub weighted_penalty: f64,
    pub max_possible_penalty: f64,
}

#[cfg(any(feature = "fmt-html", feature = "fmt-markdown"))]
impl From<&crate::types::Summary> for SummaryView {
    fn from(s: &crate::types::Summary) -> Self {
        Self {
            plugin_name: s.plugin_name.clone(),
            total_files: s.total_files,
            total_violations: s.total_violations,
            errors: s.errors,
            warnings: s.warnings,
            infos: s.infos,
            score: s.score,
            grade: s.grade.clone(),
            grade_label: s.grade_label.clone(),
            duration_ms: s.duration_ms,
            avg_file_ms: s.avg_file_ms,
            min_file_ms: s.min_file_ms,
            max_file_ms: s.max_file_ms,
            partial_coverage: s.partial_coverage,
            all_categories_json: serde_json::to_string(&s.all_categories).unwrap_or_default(),
            weighted_penalty: s.weighted_penalty,
            max_possible_penalty: s.total_active_weight,
        }
    }
}

#[cfg(any(feature = "fmt-html", feature = "fmt-markdown"))]
#[derive(Debug, Clone, Serialize)]
pub struct ViolationGroupView<'a> {
    pub index: usize,
    pub rule_id: &'a str,
    pub category: &'a str,
    pub message: &'a str,
    pub file: String,
    pub line: usize,
    pub source_code: Option<&'a str>,
    pub suggestion: Option<&'a str>,
    pub accuracy: &'a str,
    pub accuracy_note: Option<&'a str>,
    pub reference: Option<String>,
    pub is_new_category: bool,
    pub is_last_in_category: bool,
}

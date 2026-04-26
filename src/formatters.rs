#[cfg(any(feature = "fmt-html", feature = "fmt-markdown"))]
use crate::report::{MdReportTemplate, ReportTemplate, SummaryView, ViolationGroupView};
#[cfg(any(feature = "fmt-html", feature = "fmt-markdown"))]
use askama::Template;
#[cfg(any(feature = "fmt-table", feature = "fmt-terminal"))]
use colored::Colorize;
#[cfg(any(feature = "fmt-table", feature = "fmt-terminal"))]
use std::collections::HashMap;
#[cfg(feature = "fmt-table")]
use tabled::{settings::Style, Table, Tabled};

#[allow(unused_imports)]
use crate::types::{Severity, Summary, Violation};

#[cfg(feature = "fmt-json")]
pub fn format_json(violations: &[Violation], summary: &Summary, out: &mut dyn std::io::Write) {
    #[derive(serde::Serialize)]
    struct Output<'a> {
        violations: &'a [Violation],
        summary: &'a Summary,
    }
    let output = Output {
        violations,
        summary,
    };
    writeln!(out, "{}", serde_json::to_string_pretty(&output).unwrap()).ok();
}

#[cfg(feature = "fmt-yaml")]
pub fn format_yaml(violations: &[Violation], summary: &Summary, out: &mut dyn std::io::Write) {
    #[derive(serde::Serialize)]
    struct Output<'a> {
        violations: &'a [Violation],
        summary: &'a Summary,
    }
    let output = Output {
        violations,
        summary,
    };
    writeln!(out, "{}", serde_yaml::to_string(&output).unwrap()).ok();
}

#[cfg(feature = "fmt-toml")]
pub fn format_toml(violations: &[Violation], summary: &Summary, out: &mut dyn std::io::Write) {
    #[derive(serde::Serialize)]
    struct Output {
        violations: Vec<Violation>,
        summary: Summary,
    }
    let output = Output {
        violations: violations.to_vec(),
        summary: summary.clone(),
    };
    writeln!(out, "{}", toml::to_string_pretty(&output).unwrap()).ok();
}

#[cfg(feature = "fmt-html")]
pub fn format_html(violations: &[Violation], summary: &Summary, out: &mut dyn std::io::Write) {
    let report_date = current_timestamp();
    let summary_view = SummaryView::from(summary);

    let mut errors: Vec<ViolationGroupView> = Vec::new();
    let mut warnings: Vec<ViolationGroupView> = Vec::new();
    let mut infos: Vec<ViolationGroupView> = Vec::new();

    let sorted_violations = sort_violations(violations);

    let mut error_idx = 1;
    let mut warning_idx = 1;
    let mut info_idx = 1;
    let mut last_error_cat: Option<&str> = None;
    let mut last_info_cat: Option<&str> = None;

    for v in &sorted_violations {
        let is_new_cat = match v.severity {
            Severity::Error => {
                let cat = Some(v.category.as_str());
                let result = cat != last_error_cat;
                last_error_cat = cat;
                result
            }
            Severity::Warning => {
                let cat = Some(v.category.as_str());
                let result = cat != last_error_cat;
                last_error_cat = cat;
                result
            }
            Severity::Info => {
                let cat = Some(v.category.as_str());
                let result = cat != last_info_cat;
                last_info_cat = cat;
                result
            }
        };

        let group = ViolationGroupView {
            index: match v.severity {
                Severity::Error => {
                    let idx = error_idx;
                    error_idx += 1;
                    idx
                }
                Severity::Warning => {
                    let idx = warning_idx;
                    warning_idx += 1;
                    idx
                }
                Severity::Info => {
                    let idx = info_idx;
                    info_idx += 1;
                    idx
                }
            },
            rule_id: &v.rule_id,
            category: &v.category,
            message: &v.message,
            file: v.file.to_string_lossy().to_string(),
            line: v.line,
            source_code: v.source_code.as_deref(),
            suggestion: v.suggestion.as_deref(),
            accuracy: v.accuracy.as_deref().unwrap_or("approximate"),
            accuracy_note: v.accuracy_note.as_deref(),
            reference: v.reference.clone(),
            is_new_category: is_new_cat,
            is_last_in_category: false,
        };

        match v.severity {
            Severity::Error => errors.push(group),
            Severity::Warning => warnings.push(group),
            Severity::Info => infos.push(group),
        }
    }

    if let Some(last) = errors.last_mut() {
        last.is_last_in_category = true;
    }
    if let Some(last) = warnings.last_mut() {
        last.is_last_in_category = true;
    }
    if let Some(last) = infos.last_mut() {
        last.is_last_in_category = true;
    }

    let template = ReportTemplate {
        report_date: &report_date,
        summary: summary_view,
        errors,
        warnings,
        infos,
    };

    writeln!(out, "{}", template.render().unwrap()).ok();
}

#[cfg(any(feature = "fmt-html", feature = "fmt-markdown"))]
fn sort_violations(violations: &[Violation]) -> Vec<&Violation> {
    let mut sorted: Vec<&Violation> = violations.iter().collect();
    sorted.sort_by(|a, b| {
        let cat_cmp = a.category.cmp(&b.category);
        if cat_cmp != std::cmp::Ordering::Equal {
            return cat_cmp;
        }
        let sev_cmp = match (&a.severity, &b.severity) {
            (Severity::Error, Severity::Error) => std::cmp::Ordering::Equal,
            (Severity::Error, _) => std::cmp::Ordering::Less,
            (_, Severity::Error) => std::cmp::Ordering::Greater,
            (Severity::Warning, Severity::Warning) => std::cmp::Ordering::Equal,
            (Severity::Warning, Severity::Info) => std::cmp::Ordering::Less,
            (Severity::Info, Severity::Warning) => std::cmp::Ordering::Greater,
            (Severity::Info, Severity::Info) => std::cmp::Ordering::Equal,
        };
        if sev_cmp != std::cmp::Ordering::Equal {
            return sev_cmp;
        }
        a.rule_id.cmp(&b.rule_id)
    });
    sorted
}

#[cfg(any(feature = "fmt-html", feature = "fmt-markdown", feature = "fmt-table"))]
fn current_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let days = now / 86400;
    let years = 1970 + (days / 365);
    let remaining_days = days % 365;
    let month = remaining_days / 30 + 1;
    let day = remaining_days % 30 + 1;
    let secs = now % 86400;
    let hour = secs / 3600;
    let min = (secs % 3600) / 60;
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        years, month, day, hour, min
    )
}

#[cfg(feature = "fmt-markdown")]
pub fn format_markdown(violations: &[Violation], summary: &Summary, out: &mut dyn std::io::Write) {
    let report_date = current_timestamp();
    let summary_view = SummaryView::from(summary);

    let sorted_violations = sort_violations(violations);

    let mut errors: Vec<ViolationGroupView> = Vec::new();
    let mut warnings: Vec<ViolationGroupView> = Vec::new();
    let mut infos: Vec<ViolationGroupView> = Vec::new();

    for v in &sorted_violations {
        let group = ViolationGroupView {
            index: 0,
            rule_id: &v.rule_id,
            category: &v.category,
            message: &v.message,
            file: v.file.to_string_lossy().to_string(),
            line: v.line,
            source_code: v.source_code.as_deref(),
            suggestion: v.suggestion.as_deref(),
            accuracy: v.accuracy.as_deref().unwrap_or("approximate"),
            accuracy_note: v.accuracy_note.as_deref(),
            reference: v.reference.clone(),
            is_new_category: false,
            is_last_in_category: false,
        };

        match v.severity {
            Severity::Error => errors.push(group),
            Severity::Warning => warnings.push(group),
            Severity::Info => infos.push(group),
        }
    }

    let template = MdReportTemplate {
        report_date: &report_date,
        summary: summary_view,
        errors,
        warnings,
        infos,
    };

    writeln!(out, "{}", template.render().unwrap()).ok();
}

#[cfg(feature = "fmt-table")]
pub fn format_table(violations: &[Violation], summary: &Summary, out: &mut dyn std::io::Write) {
    #[derive(Tabled)]
    struct Row {
        #[tabled(rename = "Severity")]
        severity: String,
        #[tabled(rename = "Rule")]
        rule: String,
        #[tabled(rename = "Accuracy")]
        accuracy: String,
        #[tabled(rename = "File")]
        file: String,
        #[tabled(rename = "Line")]
        line: usize,
        #[tabled(rename = "Message")]
        message: String,
    }

    writeln!(out, "OPLint Compliance Report — {}", current_timestamp()).ok();

    if violations.is_empty() {
        writeln!(out, "✓ No violations found.").ok();
        print_score_line(out, summary);
        writeln!(
            out,
            "Summary: 0 errors, 0 warnings, 0 info ({} files scanned)",
            summary.total_files
        )
        .ok();
        return;
    }

    let mut sorted: Vec<&Violation> = violations.iter().collect();
    sorted.sort_by(|a, b| {
        let sev_ord = |s: &Severity| match s {
            Severity::Error => 0,
            Severity::Warning => 1,
            Severity::Info => 2,
        };
        sev_ord(&a.severity)
            .cmp(&sev_ord(&b.severity))
            .then_with(|| a.rule_id.cmp(&b.rule_id))
    });

    let rows: Vec<Row> = sorted
        .iter()
        .map(|v| {
            let file = v.file.to_string_lossy();
            let file_str = if file.len() > 45 {
                format!("…{}", &file[file.len() - 45..])
            } else {
                file.into_owned()
            };
            let acc = v.accuracy.as_deref().unwrap_or("approximate");
            let acc_colored = match acc {
                "exact" => "exact".green().dimmed().to_string(),
                _ => "approx".yellow().to_string(),
            };
            Row {
                severity: match v.severity {
                    Severity::Error => "error".red().bold().to_string(),
                    Severity::Warning => "warning".yellow().to_string(),
                    Severity::Info => "info".cyan().to_string(),
                },
                rule: v.rule_id.clone(),
                accuracy: acc_colored,
                file: file_str,
                line: v.line,
                message: v.message.clone(),
            }
        })
        .collect();

    let mut table = Table::new(rows);
    table.with(Style::sharp());
    writeln!(out, "{table}").ok();
    print_score_line(out, summary);
    print_category_breakdown(out, violations);
    writeln!(
        out,
        "Summary: {} errors, {} warnings, {} info ({} files scanned)",
        summary.errors, summary.warnings, summary.infos, summary.total_files
    )
    .ok();
    print_perf_line(out, summary);
}

#[cfg(any(feature = "fmt-table", feature = "fmt-terminal"))]
fn print_score_line(out: &mut dyn std::io::Write, summary: &Summary) {
    let score_s = summary.score.to_string();
    let grade_s = summary.grade.as_str();
    let partial_marker = if summary.partial_coverage { "*" } else { "" };
    let (score_colored, grade_colored) = match summary.score {
        90..=100 => (score_s.green().bold(), grade_s.green().bold()),
        80..=89 => (score_s.cyan().bold(), grade_s.cyan().bold()),
        70..=79 => (score_s.yellow().bold(), grade_s.yellow().bold()),
        60..=69 => (score_s.yellow(), grade_s.yellow()),
        _ => (score_s.red().bold(), grade_s.red().bold()),
    };
    writeln!(
        out,
        "Compliance: {}/100{} · [{}] {}",
        score_colored, partial_marker, grade_colored, summary.grade_label
    )
    .ok();
    if summary.partial_coverage {
        writeln!(
            out,
            "  {} Score covers a subset of guidelines — some rules are disabled in config.",
            "*".yellow()
        )
        .ok();
    }
}

#[cfg(any(feature = "fmt-table", feature = "fmt-terminal"))]
fn print_perf_line(out: &mut dyn std::io::Write, summary: &Summary) {
    writeln!(
        out,
        "Performance: {} ms total · avg {} ms/file · min {} ms · max {} ms",
        summary.duration_ms.to_string().dimmed(),
        summary.avg_file_ms.to_string().dimmed(),
        summary.min_file_ms.to_string().dimmed(),
        summary.max_file_ms.to_string().dimmed(),
    )
    .ok();
}

#[cfg(feature = "fmt-terminal")]
pub fn format_terminal(violations: &[Violation], summary: &Summary, out: &mut dyn std::io::Write) {
    for v in violations {
        let icon = match v.severity {
            Severity::Error => "✗".red().bold(),
            Severity::Warning => "⚠".yellow(),
            Severity::Info => "ℹ".cyan(),
        };

        let sev_text = match v.severity {
            Severity::Error => "ERROR".red().bold(),
            Severity::Warning => "WARN ".yellow(),
            Severity::Info => "INFO ".cyan(),
        };

        let acc_tag = match v.accuracy.as_deref().unwrap_or("approximate") {
            "exact" => " [exact]".dimmed(),
            _ => " [approx]".yellow(),
        };

        writeln!(
            out,
            "{} {} [{:<10}]{} {} at {}:{}",
            icon,
            sev_text,
            v.rule_id,
            acc_tag,
            v.message,
            v.file.display(),
            v.line
        )
        .ok();
        if let Some(ref s) = v.suggestion {
            writeln!(out, "  {} {}", "Suggestion:".dimmed(), s).ok();
        }
        if let Some(ref n) = v.accuracy_note {
            writeln!(out, "  {} {}", "Note:".dimmed(), n.dimmed()).ok();
        }
        if let Some(ref r) = v.reference {
            writeln!(out, "  {} {}", "Reference:".dimmed(), r.dimmed()).ok();
        }
    }

    print_category_breakdown(out, violations);
    print_score_line(out, summary);
    writeln!(
        out,
        "Summary: {} errors, {} warnings, {} info ({} files scanned)",
        summary.errors, summary.warnings, summary.infos, summary.total_files
    )
    .ok();
    print_perf_line(out, summary);
}

#[cfg(any(feature = "fmt-table", feature = "fmt-terminal"))]
fn print_category_breakdown(out: &mut dyn std::io::Write, violations: &[Violation]) {
    if violations.is_empty() {
        return;
    }

    let mut cat_stats: HashMap<String, (usize, usize, usize)> = HashMap::new();
    for v in violations {
        let stats = cat_stats.entry(v.category.clone()).or_insert((0, 0, 0));
        match v.severity {
            Severity::Error => stats.0 += 1,
            Severity::Warning => stats.1 += 1,
            Severity::Info => stats.2 += 1,
        }
    }

    writeln!(out, "\n{}", "Category Breakdown:".bold()).ok();
    let mut sorted_cats: Vec<_> = cat_stats.into_iter().collect();
    sorted_cats.sort_by(|a, b| b.1 .0.cmp(&a.1 .0).then_with(|| b.1 .1.cmp(&a.1 .1)));

    for (cat, (e, w, i)) in sorted_cats {
        let bar = draw_terminal_bar(e, w, i, 20);
        writeln!(
            out,
            "  {:<15} {}  {} err, {} warn, {} info",
            cat.dimmed(),
            bar,
            e,
            w,
            i
        )
        .ok();
    }
    writeln!(out).ok();
}

#[cfg(any(feature = "fmt-table", feature = "fmt-terminal"))]
fn draw_terminal_bar(e: usize, w: usize, i: usize, width: usize) -> String {
    let total = e + w + i;
    if total == 0 {
        return ".".dimmed().to_string().repeat(width);
    }

    let e_w = ((e as f64 / total as f64) * width as f64).round() as usize;
    let w_w = ((w as f64 / total as f64) * width as f64).round() as usize;
    let i_w = ((i as f64 / total as f64) * width as f64).round() as usize;

    let current_total = e_w + w_w + i_w;
    let (mut e_w, mut w_w, mut i_w) = (e_w, w_w, i_w);
    if current_total > width {
        let diff = current_total - width;
        let r_i = i_w.min(diff);
        i_w -= r_i;
        let diff = diff - r_i;
        let r_w = w_w.min(diff);
        w_w -= r_w;
        let diff = diff - r_w;
        e_w -= diff;
    } else if current_total < width && total > 0 {
        let diff = width - current_total;
        if e > 0 {
            e_w += diff;
        } else if w > 0 {
            w_w += diff;
        } else {
            i_w += diff;
        }
    }

    format!(
        "{}{}{}",
        "■".repeat(e_w).red(),
        "■".repeat(w_w).yellow(),
        "■".repeat(i_w).cyan()
    )
}

use oplint::types::{Severity, Summary, Violation};
use std::path::PathBuf;

#[test]
fn test_severity_conversion() {
    assert_eq!(Severity::Error.as_str(), "error");
    assert_eq!(Severity::parse("WARNING"), Some(Severity::Warning));
    assert_eq!(Severity::parse("invalid"), None);
}

#[test]
fn test_violation_builder() {
    let v = Violation::new(
        "ID",
        "CAT",
        "MSG",
        Severity::Info,
        PathBuf::from("f.ts"),
        10,
    )
    .with_column(5)
    .with_suggestion("SUG")
    .with_source_code("SRC")
    .with_accuracy("ACC", Some("NOTE"));

    assert_eq!(v.rule_id, "ID");
    assert_eq!(v.column, Some(5));
    assert_eq!(v.suggestion, Some("SUG".to_string()));
    assert_eq!(v.accuracy, Some("ACC".to_string()));
    assert_eq!(v.accuracy_note, Some("NOTE".to_string()));
}

fn make_violation(rule_id: &str, file: &str, severity: Severity) -> Violation {
    Violation::new(rule_id, "Test", "msg", severity, PathBuf::from(file), 1)
}

#[test]
fn test_summary_scoring_zero_violations() {
    // No violations → perfect score
    let mut s = Summary::new();
    s.set_rule_baseline(100.0);
    s.finalize(&[]);
    assert_eq!(s.score, 100);
    assert_eq!(s.grade, "A+");
    assert_eq!(s.grade_label, "Perfect");
}

#[test]
fn test_summary_scoring_one_error() {
    // 1 error rule, fires once on one file
    // penalty = 10 × (1 + log2(1)) = 10 × 1.0 = 10
    // score   = round(100 × (1 - 10/100)) = 90  →  A
    let mut s = Summary::new();
    s.set_rule_baseline(100.0);
    s.add_violation(&make_violation("E001", "a.ts", Severity::Error));
    s.finalize(&[make_violation("E001", "a.ts", Severity::Error)]);
    assert_eq!(s.score, 90);
    assert_eq!(s.grade, "A");
}

#[test]
fn test_summary_scoring_concentrated_errors() {
    // Same rule fires 4× in one file
    // penalty = 10 × (1 + log2(4)) = 10 × 3.0 = 30
    // score   = round(100 × (1 - 30/100)) = 70  →  C
    let violations: Vec<Violation> = (0..4)
        .map(|_| make_violation("E001", "a.ts", Severity::Error))
        .collect();
    let mut s = Summary::new();
    s.set_rule_baseline(100.0);
    for v in &violations {
        s.add_violation(v);
    }
    s.finalize(&violations);
    assert_eq!(s.score, 70);
    assert_eq!(s.grade, "C");
}

#[test]
fn test_summary_scoring_spread_vs_concentrated() {
    // Same rule fires once in 4 different files → max_occ = 1
    // penalty = 10 × (1 + log2(1)) = 10
    // score   = 90  →  A
    let spread: Vec<Violation> = ["a.ts", "b.ts", "c.ts", "d.ts"]
        .iter()
        .map(|f| make_violation("E001", f, Severity::Error))
        .collect();
    let mut s = Summary::new();
    s.set_rule_baseline(100.0);
    for v in &spread {
        s.add_violation(v);
    }
    s.finalize(&spread);
    assert_eq!(s.score, 90); // same as 1 occurrence — spread doesn't amplify

    // Same rule fires 4× in one file → max_occ = 4
    // penalty = 10 × 3.0 = 30 → score 70
    let concentrated: Vec<Violation> = (0..4)
        .map(|_| make_violation("E001", "a.ts", Severity::Error))
        .collect();
    let mut s2 = Summary::new();
    s2.set_rule_baseline(100.0);
    for v in &concentrated {
        s2.add_violation(v);
    }
    s2.finalize(&concentrated);
    assert_eq!(s2.score, 70); // concentrated is worse
}

#[test]
fn test_summary_scoring_warning() {
    // 1 warning rule fires once
    // penalty = 5 × (1 + log2(1)) = 5
    // score   = round(100 × (1 - 5/100)) = 95  →  A
    let v = make_violation("W001", "a.ts", Severity::Warning);
    let mut s = Summary::new();
    s.set_rule_baseline(100.0);
    s.add_violation(&v);
    s.finalize(&[v]);
    assert_eq!(s.score, 95);
    assert_eq!(s.grade, "A");
}

#[test]
fn test_summary_timing() {
    let mut s = Summary::new();
    s.set_timing(100, &[10, 20, 30]);
    assert_eq!(s.duration_ms, 100);
    assert_eq!(s.min_file_ms, 10);
    assert_eq!(s.max_file_ms, 30);
    assert_eq!(s.avg_file_ms, 20);
}

#[test]
fn partial_coverage_defaults_false() {
    let s = Summary::new();
    assert!(!s.partial_coverage);
}

#[test]
fn partial_coverage_survives_finalize() {
    let mut s = Summary::new();
    s.total_files = 1;
    s.partial_coverage = true;
    s.finalize(&[]);
    assert!(s.partial_coverage);
    assert!(s.score <= 100);
}

#[test]
fn partial_coverage_false_not_affected_by_finalize() {
    let mut s = Summary::new();
    s.total_files = 1;
    s.finalize(&[]);
    assert!(!s.partial_coverage);
}

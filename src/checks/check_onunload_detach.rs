use super::CheckContext;
use crate::types::Violation;
use std::path::Path;
use tree_sitter::{Query, QueryCursor, StreamingIterator};

pub fn check(path: &Path, content: &str, ctx: &CheckContext<'_>) -> Vec<Violation> {
    let Some((language, tree)) = super::ts_parse(content) else {
        return Vec::new();
    };
    let root = tree.root_node();

    let Ok(onunload_query) = Query::new(
        &language,
        r#"
        (method_definition
          name: (property_identifier) @method_name
          (#eq? @method_name "onunload"))
        "#,
    ) else {
        return Vec::new();
    };

    let Some(method_name_idx) = super::capture_index(&onunload_query, "method_name") else {
        return Vec::new();
    };

    let mut onunload_ranges: Vec<(usize, usize)> = Vec::new();
    let mut cursor = QueryCursor::new();

    let mut matches = cursor.matches(&onunload_query, root, content.as_bytes());
    while let Some(m) = matches.next() {
        for capture in m.captures.iter().filter(|c| c.index == method_name_idx) {
            let mut node = capture.node;
            loop {
                if node.kind() == "method_definition" {
                    onunload_ranges.push((node.start_byte(), node.end_byte()));
                    break;
                }
                match node.parent() {
                    Some(p) => node = p,
                    None => break,
                }
            }
        }
    }

    if onunload_ranges.is_empty() {
        return vec![];
    }

    let Ok(detach_query) = Query::new(
        &language,
        r#"
        (call_expression
          function: (member_expression
            property: (property_identifier) @method
            (#eq? @method "detach")))
        "#,
    ) else {
        return Vec::new();
    };

    let Some(method_idx) = super::capture_index(&detach_query, "method") else {
        return Vec::new();
    };

    let mut cursor = QueryCursor::new();
    let mut violations = Vec::new();

    let mut matches = cursor.matches(&detach_query, root, content.as_bytes());
    while let Some(m) = matches.next() {
        for capture in m.captures.iter().filter(|c| c.index == method_idx) {
            let node = capture.node;
            let start = node.start_byte();
            let end = node.end_byte();

            if onunload_ranges
                .iter()
                .any(|(s, e)| start >= *s && end <= *e)
            {
                let row = node.start_position().row;
                let line = row + 1;
                let current = content.lines().nth(row).unwrap_or("").trim().to_string();
                let mut v = Violation::new(
                    ctx.rule_id,
                    ctx.rule_category,
                    ctx.rule_message,
                    ctx.rule_severity.clone(),
                    path.to_path_buf(),
                    line,
                );
                if let Some(s) = ctx.rule_suggestion {
                    v = v.with_suggestion(s);
                }
                v = v.with_source_code(&current);
                v = v.with_accuracy(ctx.accuracy, ctx.accuracy_note);
                violations.push(v);
            }
        }
    }

    violations
}

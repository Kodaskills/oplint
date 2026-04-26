use super::CheckContext;
use crate::types::Violation;
use std::path::Path;
use tree_sitter::{Query, QueryCursor, StreamingIterator};

/// API006: Flag any class that extends `PopoverSuggest`, the deprecated
/// suggest base class. It should be replaced with `AbstractInputSuggest`.
pub fn check(path: &Path, content: &str, ctx: &CheckContext<'_>) -> Vec<Violation> {
    let Some((language, tree)) = super::ts_parse(content) else {
        return Vec::new();
    };

    let query_str = r#"
        (class_declaration
          (class_heritage
            (extends_clause
              value: (identifier) @superclass
              (#eq? @superclass "PopoverSuggest"))))
    "#;
    let Ok(query) = Query::new(&language, query_str) else {
        return Vec::new();
    };
    let Some(class_idx) = super::capture_index(&query, "superclass") else {
        return Vec::new();
    };

    let mut cursor = QueryCursor::new();
    let mut violations = Vec::new();

    let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());
    while let Some(m) = matches.next() {
        for capture in m.captures.iter().filter(|c| c.index == class_idx) {
            let node = capture.node;
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

    violations
}

use super::CheckContext;
use crate::types::Violation;
use std::path::Path;
use tree_sitter::{Query, QueryCursor, StreamingIterator};

/// MOB001: Flag regex lookbehind assertions (`(?<=` / `(?<!`) unsupported on some iOS versions.
pub fn check(path: &Path, content: &str, ctx: &CheckContext<'_>) -> Vec<Violation> {
    let Some((language, tree)) = super::ts_parse(content) else {
        return Vec::new();
    };

    let query_str = r#"
        [
            (regex (regex_pattern) @pattern)
            (new_expression
                constructor: (identifier) @ctor
                (#eq? @ctor "RegExp")
                arguments: (arguments (string (string_fragment) @pattern)))
        ]
    "#;
    let Ok(query) = Query::new(&language, query_str) else {
        return Vec::new();
    };
    let Some(pattern_idx) = super::capture_index(&query, "pattern") else {
        return Vec::new();
    };

    let mut cursor = QueryCursor::new();
    let mut violations = Vec::new();

    let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());
    while let Some(m) = matches.next() {
        for capture in m.captures.iter().filter(|c| c.index == pattern_idx) {
            let node = capture.node;
            if let Ok(pattern) = node.utf8_text(content.as_bytes()) {
                if pattern.contains("(?<=") || pattern.contains("(?<!") {
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
    }

    violations
}

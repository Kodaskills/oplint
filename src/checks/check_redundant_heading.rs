use super::CheckContext;
use crate::types::Violation;
use std::collections::HashSet;
use std::path::Path;
use tree_sitter::{Query, QueryCursor, StreamingIterator};

/// UI004: Flag well-known redundant heading strings passed to `setName()`.
///
/// The guideline warns against single‑section headings like “General” or
/// “Settings”. Because we can't detect the number of sections automatically,
/// the rule simply flags these common strings for manual review.
pub fn check(path: &Path, content: &str, ctx: &CheckContext<'_>) -> Vec<Violation> {
    let Some((language, tree)) = super::ts_parse(content) else {
        return Vec::new();
    };

    // The strings that typically indicate a redundant top‑level heading.
    let redundant_strings: HashSet<&str> =
        ["General", "Settings", "Options"].iter().copied().collect();

    // Query: capture the literal string in `setName()` calls.
    let query_str = r#"
        (call_expression
          function: (member_expression
            property: (property_identifier) @method
            (#eq? @method "setName"))
          arguments: (arguments
            (string
              (string_fragment) @text)))
    "#;
    let Ok(query) = Query::new(&language, query_str) else {
        return Vec::new();
    };
    let Some(text_idx) = super::capture_index(&query, "text") else {
        return Vec::new();
    };

    let mut cursor = QueryCursor::new();
    let mut violations = Vec::new();

    let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());
    while let Some(m) = matches.next() {
        for capture in m.captures.iter().filter(|c| c.index == text_idx) {
            let node = capture.node;
            if let Ok(text) = node.utf8_text(content.as_bytes()) {
                if redundant_strings.contains(text) {
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

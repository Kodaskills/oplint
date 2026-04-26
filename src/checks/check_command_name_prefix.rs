use super::CheckContext;
use crate::types::Violation;
use std::path::Path;
use tree_sitter::{Query, QueryCursor, StreamingIterator};

pub fn check(path: &Path, content: &str, ctx: &CheckContext<'_>) -> Vec<Violation> {
    let plugin_name = match ctx.plugin_name {
        Some(n) => n,
        None => return vec![],
    };

    let Some((language, tree)) = super::ts_parse(content) else {
        return Vec::new();
    };

    let query_str = r#"
        (call_expression
          function: (member_expression
            property: (property_identifier) @method
            (#match? @method "^(addCommand|addCommandIf)$"))
          arguments: (arguments
            (object
              (pair
                key: (property_identifier) @key
                (#eq? @key "name")
                value: (string
                  (string_fragment) @cmd_name)))))
    "#;
    let Ok(query) = Query::new(&language, query_str) else {
        return Vec::new();
    };

    let Some(cmd_name_idx) = super::capture_index(&query, "cmd_name") else {
        return Vec::new();
    };

    let prefix_lower = plugin_name.to_lowercase();
    let mut cursor = QueryCursor::new();
    let mut violations = Vec::new();

    let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());
    while let Some(m) = matches.next() {
        for capture in m.captures.iter().filter(|c| c.index == cmd_name_idx) {
            let node = capture.node;
            if let Ok(cmd_name) = node.utf8_text(content.as_bytes()) {
                if cmd_name.to_lowercase().starts_with(&prefix_lower) {
                    let cleaned = cmd_name[plugin_name.len()..].trim_start_matches(':').trim();
                    let row = node.start_position().row;
                    let line = row + 1;
                    let current = content.lines().nth(row).unwrap_or("").trim().to_string();
                    let msg = format!(
                        "Command name '{}' starts with the plugin name. Use '{}' instead.",
                        cmd_name, cleaned
                    );
                    let mut v = Violation::new(
                        ctx.rule_id,
                        ctx.rule_category,
                        &msg,
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

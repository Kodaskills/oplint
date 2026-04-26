use super::CheckContext;
use crate::types::Violation;
use std::path::Path;
use tree_sitter::{Query, QueryCursor, StreamingIterator};

pub fn check(path: &Path, content: &str, ctx: &CheckContext<'_>) -> Vec<Violation> {
    let plugin_id = match ctx.plugin_id {
        Some(id) => id,
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
                (#eq? @key "id")
                value: (string
                  (string_fragment) @cmd_id)))))
    "#;
    let Ok(query) = Query::new(&language, query_str) else {
        return Vec::new();
    };

    let Some(cmd_id_idx) = super::capture_index(&query, "cmd_id") else {
        return Vec::new();
    };

    let mut cursor = QueryCursor::new();
    let mut violations = Vec::new();

    let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());
    while let Some(m) = matches.next() {
        for capture in m.captures.iter().filter(|c| c.index == cmd_id_idx) {
            let node = capture.node;
            if let Ok(cmd_id) = node.utf8_text(content.as_bytes()) {
                let prefix = format!("{}:", plugin_id);
                let prefix_spaced = format!("{} :", plugin_id);

                if cmd_id.starts_with(&prefix) || cmd_id.starts_with(&prefix_spaced) {
                    let cleaned = if cmd_id.starts_with(&prefix) {
                        cmd_id[prefix.len()..].trim_start()
                    } else {
                        cmd_id[prefix_spaced.len()..].trim_start()
                    };

                    let row = node.start_position().row;
                    let line = row + 1;
                    let current = content.lines().nth(row).unwrap_or("").trim().to_string();
                    let msg = format!(
                        "Command ID '{}' includes the plugin ID prefix. Use '{}' instead.",
                        cmd_id, cleaned
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

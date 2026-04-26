use super::CheckContext;
use crate::types::Violation;
use std::path::Path;
use tree_sitter::{Node, Query, QueryCursor, StreamingIterator};

pub fn check(path: &Path, content: &str, ctx: &CheckContext<'_>) -> Vec<Violation> {
    let Some((language, tree)) = super::ts_parse(content) else {
        return Vec::new();
    };
    let root = tree.root_node();

    let query_str = r#"
        (call_expression
          function: (member_expression
            property: (property_identifier) @method
            (#match? @method "^(addCommand|addCommandIf)$"))
          arguments: (arguments
            (object
              (pair
                key: (property_identifier) @cb_key
                (#match? @cb_key "^(callback|editorCallback|checkCallback|editorCheckCallback)$")
                value: _ @cb_value))))
    "#;
    let Ok(query) = Query::new(&language, query_str) else {
        return Vec::new();
    };

    let Some(key_idx) = super::capture_index(&query, "cb_key") else {
        return Vec::new();
    };
    let Some(value_idx) = super::capture_index(&query, "cb_value") else {
        return Vec::new();
    };

    let mut cursor = QueryCursor::new();
    let mut violations = Vec::new();

    let mut matches = cursor.matches(&query, root, content.as_bytes());
    while let Some(m) = matches.next() {
        let key_node = m
            .captures
            .iter()
            .find(|c| c.index == key_idx)
            .map(|c| c.node);
        let value_node = m
            .captures
            .iter()
            .find(|c| c.index == value_idx)
            .map(|c| c.node);

        let (key_node, value_node): (_, _) = match (key_node, value_node) {
            (Some(k), Some(v)) => (k, v),
            _ => continue,
        };

        let key = key_node.utf8_text(content.as_bytes()).unwrap_or("");

        let func_node = match value_node.kind() {
            "arrow_function" | "function_expression" => value_node,
            _ => continue,
        };

        let has_editor_param = has_parameter_named(content, func_node, "editor");

        let expect_editor = matches!(key, "editorCallback" | "editorCheckCallback");

        if expect_editor != has_editor_param {
            let row = key_node.start_position().row;
            let line = row + 1;
            let current = content.lines().nth(row).unwrap_or("").trim().to_string();
            let msg = if expect_editor {
                format!(
                    "'{}' expects a callback with an 'editor' parameter, but none was found.",
                    key
                )
            } else {
                format!(
                    "'{}' should not have an 'editor' parameter. Use 'editorCallback' instead.",
                    key
                )
            };
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

    violations
}

fn has_parameter_named(source: &str, func_node: Node, name: &str) -> bool {
    let Some(params_node) = func_node.child_by_field_name("parameters") else {
        return false;
    };

    let mut cursor = params_node.walk();
    for param_node in params_node.children(&mut cursor) {
        let param_name = if param_node.kind() == "identifier" {
            param_node.utf8_text(source.as_bytes()).ok()
        } else if param_node.kind() == "required_parameter" {
            param_node
                .child(0)
                .and_then(|c| c.utf8_text(source.as_bytes()).ok())
        } else {
            continue;
        };

        if param_name == Some(name) {
            return true;
        }
    }
    false
}

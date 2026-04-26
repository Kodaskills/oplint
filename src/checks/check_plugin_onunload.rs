use super::CheckContext;
use crate::types::Violation;
use std::path::Path;
use tree_sitter::{Query, QueryCursor, StreamingIterator};

pub fn check(path: &Path, content: &str, ctx: &CheckContext<'_>) -> Vec<Violation> {
    let Some((language, tree)) = super::ts_parse(content) else {
        return Vec::new();
    };
    let root = tree.root_node();

    let Ok(class_query) = Query::new(
        &language,
        r#"
        (class_declaration
          (class_heritage
            (extends_clause
              value: (identifier) @base
              (#eq? @base "Plugin")))
          body: (class_body) @body
          ) @class
        "#,
    ) else {
        return Vec::new();
    };

    let Some(class_idx) = super::capture_index(&class_query, "class") else {
        return Vec::new();
    };
    let Some(body_idx) = super::capture_index(&class_query, "body") else {
        return Vec::new();
    };

    let mut cursor = QueryCursor::new();
    let mut plugin_class_found = false;
    let mut onunload_found = false;
    let mut line_number = 1;

    let mut matches = cursor.matches(&class_query, root, content.as_bytes());
    while let Some(m) = matches.next() {
        for capture in m.captures.iter().filter(|c| c.index == class_idx) {
            plugin_class_found = true;
            line_number = capture.node.start_position().row + 1;
        }
        for capture in m.captures.iter().filter(|c| c.index == body_idx) {
            let class_body = capture.node;
            for child in class_body.children(&mut class_body.walk()) {
                if child.kind() == "method_definition" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        if name_node.kind() == "property_identifier" {
                            if let Ok(name) = name_node.utf8_text(content.as_bytes()) {
                                if name == "onunload" {
                                    onunload_found = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if plugin_class_found && !onunload_found {
        let mut v = Violation::new(
            ctx.rule_id,
            ctx.rule_category,
            ctx.rule_message,
            ctx.rule_severity.clone(),
            path.to_path_buf(),
            line_number,
        );
        if let Some(s) = ctx.rule_suggestion {
            v = v.with_suggestion(s);
        }
        v = v.with_accuracy(ctx.accuracy, ctx.accuracy_note);
        return vec![v];
    }

    vec![]
}

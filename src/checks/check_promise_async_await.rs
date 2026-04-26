use super::CheckContext;
use crate::types::Violation;
use std::path::Path;
use tree_sitter::{Node, Query, QueryCursor, StreamingIterator};

/// TS002: Check for new Promise() usage that isn't a promisify pattern
pub fn check(path: &Path, content: &str, ctx: &CheckContext<'_>) -> Vec<Violation> {
    let Some((language, tree)) = super::ts_parse(content) else {
        return Vec::new();
    };

    let query_str = r#"
        (new_expression
            constructor: (identifier) @constructor
            (#eq? @constructor "Promise"))
    "#;

    let Ok(query) = Query::new(&language, query_str) else {
        eprintln!("[oplint] invalid {} tree-sitter query", ctx.rule_id);
        return Vec::new();
    };

    let Some(constructor_idx) = super::capture_index(&query, "constructor") else {
        return Vec::new();
    };

    let mut cursor = QueryCursor::new();
    let mut violations = Vec::new();

    let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());
    while let Some(m) = matches.next() {
        let Some(constructor_node) = m
            .captures
            .iter()
            .find(|c| c.index == constructor_idx)
            .map(|c| c.node)
        else {
            continue;
        };

        let Some(new_expr_node) = constructor_node.parent() else {
            continue;
        };

        if let Some((resolve_name, reject_name, body_node)) =
            find_executor_in_new_expr(content, new_expr_node)
        {
            if !is_promisify_pattern(content, body_node, &resolve_name, reject_name.as_deref()) {
                let row = constructor_node.start_position().row;
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

fn find_executor_in_new_expr<'a>(
    content: &'a str,
    new_expr_node: Node<'a>,
) -> Option<(String, Option<String>, Node<'a>)> {
    let args_node = new_expr_node.child_by_field_name("arguments")?;

    let mut cursor = args_node.walk();
    for child in args_node.children(&mut cursor) {
        if child.kind() == "arrow_function" || child.kind() == "function_expression" {
            let params_node = child.child_by_field_name("parameters")?;
            let body_node = child.child_by_field_name("body")?;

            let mut param_names = Vec::new();
            let mut param_cursor = params_node.walk();
            for param in params_node.children(&mut param_cursor) {
                let name_node = if param.kind() == "identifier" {
                    Some(param)
                } else if matches!(param.kind(), "required_parameter" | "optional_parameter") {
                    param
                        .child_by_field_name("pattern")
                        .filter(|n| n.kind() == "identifier")
                } else {
                    None
                };
                if let Some(n) = name_node {
                    if let Ok(name) = n.utf8_text(content.as_bytes()) {
                        param_names.push(name.to_string());
                    }
                }
            }

            if param_names.is_empty() {
                return None;
            }

            let resolve_name = param_names[0].clone();
            let reject_name = param_names.get(1).cloned();

            return Some((resolve_name, reject_name, body_node));
        }
    }
    None
}

fn is_promisify_pattern(
    content: &str,
    body_node: Node,
    resolve_name: &str,
    reject_name: Option<&str>,
) -> bool {
    let content_bytes = content.as_bytes();
    // (node, inside_nested_fn): resolve/reject called directly inside a nested
    // callback counts as promisify; called at the top level does not.
    let mut stack: Vec<(Node, bool)> = vec![(body_node, false)];

    while let Some((node, inside_nested)) = stack.pop() {
        let is_nested_fn = matches!(
            node.kind(),
            "arrow_function" | "function_expression" | "function_declaration"
        );

        if node.kind() == "call_expression" {
            let func_text = node
                .child_by_field_name("function")
                .and_then(|f| f.utf8_text(content_bytes).ok());
            let is_resolve_or_reject =
                func_text.is_some_and(|name| name == resolve_name || Some(name) == reject_name);

            if is_resolve_or_reject {
                if inside_nested {
                    return true;
                }
            } else if is_call_with_param(content, node, resolve_name)
                || reject_name.is_some_and(|name| is_call_with_param(content, node, name))
            {
                return true;
            }
        }

        let child_inside_nested = inside_nested || is_nested_fn;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            stack.push((child, child_inside_nested));
        }
    }
    false
}

fn is_call_with_param(content: &str, node: Node, param_name: &str) -> bool {
    if node.kind() != "call_expression" {
        return false;
    }

    let Some(args_node) = node.child_by_field_name("arguments") else {
        return false;
    };

    let mut cursor = args_node.walk();
    for arg in args_node.children(&mut cursor) {
        if arg.kind() == "identifier" {
            if let Ok(name) = arg.utf8_text(content.as_bytes()) {
                if name == param_name {
                    return true;
                }
            }
        }
    }
    false
}

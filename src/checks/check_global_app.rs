use super::CheckContext;
use crate::types::Violation;
use std::path::Path;
use tree_sitter::{Node, Query, QueryCursor, StreamingIterator};

const FUNCTION_KINDS: &[&str] = &[
    "function_declaration",
    "function_expression",
    "generator_function_declaration",
    "generator_function",
    "arrow_function",
    "method_definition",
];

pub fn check(path: &Path, content: &str, ctx: &CheckContext<'_>) -> Vec<Violation> {
    let Some((language, tree)) = super::ts_parse(content) else {
        return Vec::new();
    };

    let query_str = r#"(member_expression object: (identifier) @o)"#;
    let Ok(query) = Query::new(&language, query_str) else {
        return Vec::new();
    };
    let Some(o_idx) = super::capture_index(&query, "o") else {
        return Vec::new();
    };

    let content_bytes = content.as_bytes();
    let mut cursor = QueryCursor::new();
    let mut violations = Vec::new();

    let mut matches = cursor.matches(&query, tree.root_node(), content_bytes);
    while let Some(m) = matches.next() {
        for capture in m.captures.iter().filter(|c| c.index == o_idx) {
            let node = capture.node;
            if node.utf8_text(content_bytes).ok() != Some("app") {
                continue;
            }
            if app_is_param_in_scope(&node, content_bytes) {
                continue;
            }
            if app_is_derived_from_param_in_scope(&node, content_bytes) {
                continue;
            }
            let row = node.start_position().row;
            let current = content.lines().nth(row).unwrap_or("").trim().to_string();
            let mut v = Violation::new(
                ctx.rule_id,
                ctx.rule_category,
                ctx.rule_message,
                ctx.rule_severity.clone(),
                path.to_path_buf(),
                row + 1,
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

/// Returns true if any ancestor function/method/arrow has `app` as a named parameter.
fn app_is_param_in_scope(node: &Node<'_>, content_bytes: &[u8]) -> bool {
    let mut current = node.parent();
    while let Some(ancestor) = current {
        if FUNCTION_KINDS.contains(&ancestor.kind()) && func_has_app_param(&ancestor, content_bytes)
        {
            return true;
        }
        current = ancestor.parent();
    }
    false
}

fn func_has_app_param(func_node: &Node<'_>, content_bytes: &[u8]) -> bool {
    let Some(params) = func_node.child_by_field_name("parameters") else {
        return false;
    };
    match params.kind() {
        "formal_parameters" => formal_params_has_app(&params, content_bytes),
        // Single-param arrow: `app => app.vault`
        "identifier" => params.utf8_text(content_bytes).ok() == Some("app"),
        _ => false,
    }
}

fn formal_params_has_app(params_node: &Node<'_>, content_bytes: &[u8]) -> bool {
    let mut cursor = params_node.walk();
    for child in params_node.children(&mut cursor) {
        match child.kind() {
            // Plain JS: function foo(app) {}
            "identifier" => {
                if child.utf8_text(content_bytes).ok() == Some("app") {
                    return true;
                }
            }
            // TypeScript: function foo(app: App) {} or constructor(private app: App)
            "required_parameter" | "optional_parameter" => {
                if param_is_app(&child, content_bytes) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn param_is_app(param_node: &Node<'_>, content_bytes: &[u8]) -> bool {
    // Named field "pattern" holds the identifier in TypeScript params
    if let Some(pattern) = param_node.child_by_field_name("pattern") {
        if pattern.kind() == "identifier" {
            return pattern.utf8_text(content_bytes).ok() == Some("app");
        }
    }
    // Fallback: first identifier child (covers simpler forms)
    let mut cursor = param_node.walk();
    for child in param_node.children(&mut cursor) {
        if child.kind() == "identifier" {
            return child.utf8_text(content_bytes).ok() == Some("app");
        }
    }
    false
}

/// Returns true if `app` is a local variable derived from a function parameter,
/// e.g. `const app = ea.plugin.app` where `ea` is a param of the enclosing function.
fn app_is_derived_from_param_in_scope(node: &Node<'_>, content_bytes: &[u8]) -> bool {
    let mut current = node.parent();
    while let Some(ancestor) = current {
        if FUNCTION_KINDS.contains(&ancestor.kind()) {
            if let Some(root) = find_app_var_init_root(&ancestor, content_bytes) {
                if func_has_param_named(&ancestor, &root, content_bytes) {
                    return true;
                }
            }
        }
        current = ancestor.parent();
    }
    false
}

/// Scan direct statements in a function body for `const/let/var app = <expr>`
/// and return the leftmost root identifier of the RHS, if any.
fn find_app_var_init_root(func_node: &Node<'_>, content_bytes: &[u8]) -> Option<String> {
    let body = func_node.child_by_field_name("body")?;
    let mut cursor = body.walk();
    for stmt in body.children(&mut cursor) {
        if let Some(root) = app_init_root_in_decl(&stmt, content_bytes) {
            return Some(root);
        }
    }
    None
}

fn app_init_root_in_decl(node: &Node<'_>, content_bytes: &[u8]) -> Option<String> {
    match node.kind() {
        "lexical_declaration" | "variable_declaration" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() != "variable_declarator" {
                    continue;
                }
                let Some(name) = child.child_by_field_name("name") else {
                    continue;
                };
                if name.utf8_text(content_bytes).ok() != Some("app") {
                    continue;
                }
                let Some(value) = child.child_by_field_name("value") else {
                    continue;
                };
                return member_expr_root(&value, content_bytes);
            }
            None
        }
        _ => None,
    }
}

/// Walk a member_expression chain to the leftmost root identifier.
fn member_expr_root(node: &Node<'_>, content_bytes: &[u8]) -> Option<String> {
    match node.kind() {
        "identifier" => node.utf8_text(content_bytes).ok().map(str::to_string),
        "member_expression" => {
            let obj = node.child_by_field_name("object")?;
            member_expr_root(&obj, content_bytes)
        }
        _ => None,
    }
}

/// Check if a function has a parameter with the given name.
fn func_has_param_named(func_node: &Node<'_>, name: &str, content_bytes: &[u8]) -> bool {
    let Some(params) = func_node.child_by_field_name("parameters") else {
        return false;
    };
    match params.kind() {
        "formal_parameters" => formal_params_has_name(&params, name, content_bytes),
        "identifier" => params.utf8_text(content_bytes).ok() == Some(name),
        _ => false,
    }
}

fn formal_params_has_name(params_node: &Node<'_>, name: &str, content_bytes: &[u8]) -> bool {
    let mut cursor = params_node.walk();
    for child in params_node.children(&mut cursor) {
        match child.kind() {
            "identifier" => {
                if child.utf8_text(content_bytes).ok() == Some(name) {
                    return true;
                }
            }
            "required_parameter" | "optional_parameter" => {
                if param_has_name(&child, name, content_bytes) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn param_has_name(param_node: &Node<'_>, name: &str, content_bytes: &[u8]) -> bool {
    if let Some(pattern) = param_node.child_by_field_name("pattern") {
        if pattern.kind() == "identifier" {
            return pattern.utf8_text(content_bytes).ok() == Some(name);
        }
    }
    let mut cursor = param_node.walk();
    for child in param_node.children(&mut cursor) {
        if child.kind() == "identifier" {
            return child.utf8_text(content_bytes).ok() == Some(name);
        }
    }
    false
}

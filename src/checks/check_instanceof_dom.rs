use super::CheckContext;
use crate::types::Violation;
use std::collections::HashSet;
use std::path::Path;
use tree_sitter::{Query, QueryCursor, StreamingIterator};

/// Exact set of DOM/UIEvent constructors that are commonly used in Obsidian
/// plugins and should use `.instanceOf()` for cross‑window safety.
const DOM_UI_CONSTRUCTORS: &[&str] = &[
    // HTML element types
    "HTMLElement",
    "HTMLDivElement",
    "HTMLSpanElement",
    "HTMLInputElement",
    "HTMLTextAreaElement",
    "HTMLButtonElement",
    "HTMLSelectElement",
    "HTMLOptionElement",
    "HTMLAnchorElement",
    "HTMLImageElement",
    "HTMLCanvasElement",
    "HTMLVideoElement",
    "HTMLAudioElement",
    "HTMLTableElement",
    "HTMLTableRowElement",
    "HTMLTableCellElement",
    "HTMLUListElement",
    "HTMLOListElement",
    "HTMLLIElement",
    "HTMLParagraphElement",
    "HTMLHeadingElement",
    "HTMLPreElement",
    "HTMLBRElement",
    "HTMLHRElement",
    "HTMLFormElement",
    "HTMLLabelElement",
    "HTMLDialogElement",
    "HTMLDetailsElement",
    "HTMLSummaryElement",
    // Generic element types
    "Element",
    "Node",
    "Document",
    "DocumentFragment",
    "ShadowRoot",
    "Text",
    "Comment",
    // SVG types (occasionally used)
    "SVGElement",
    "SVGSVGElement",
    "SVGPathElement",
    "SVGCircleElement",
    "SVGRectElement",
    "SVGLineElement",
    "SVGTextElement",
    // Event types
    "UIEvent",
    "MouseEvent",
    "KeyboardEvent",
    "WheelEvent",
    "FocusEvent",
    "InputEvent",
    "CompositionEvent",
    "DragEvent",
    "ClipboardEvent",
    "TouchEvent",
    "PointerEvent",
    "TransitionEvent",
    "AnimationEvent",
    "ErrorEvent",
    "CustomEvent",
    "Event",
];

/// API004: Flag `instanceof X` where X is a known DOM/UIEvent constructor,
/// and suggest `.instanceOf()` instead.
pub fn check(path: &Path, content: &str, ctx: &CheckContext<'_>) -> Vec<Violation> {
    let Some((language, tree)) = super::ts_parse(content) else {
        return Vec::new();
    };

    // Build a HashSet for O(1) lookups.
    let known_types: HashSet<&str> = DOM_UI_CONSTRUCTORS.iter().copied().collect();

    // Query: binary_expression with operator "instanceof" and an identifier on the right.
    let query_str = r#"
        (binary_expression
          operator: "instanceof"
          right: (identifier) @type_name)
    "#;
    let Ok(query) = Query::new(&language, query_str) else {
        return Vec::new();
    };
    let Some(type_idx) = super::capture_index(&query, "type_name") else {
        return Vec::new();
    };

    let mut cursor = QueryCursor::new();
    let mut violations = Vec::new();

    let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());
    while let Some(m) = matches.next() {
        for capture in m.captures.iter().filter(|c| c.index == type_idx) {
            let node = capture.node;
            if let Ok(type_name) = node.utf8_text(content.as_bytes()) {
                if known_types.contains(type_name) {
                    let row = node.start_position().row;
                    let line = row + 1;
                    let current = content.lines().nth(row).unwrap_or("").trim().to_string();
                    let message = format!(
                        "Prefer `element.instanceOf({})` over `instanceof {}` for cross‑window compatibility.",
                        type_name, type_name
                    );
                    let mut v = Violation::new(
                        ctx.rule_id,
                        ctx.rule_category,
                        &message,
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

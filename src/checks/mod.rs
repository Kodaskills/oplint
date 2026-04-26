pub mod check_callback_signature;
pub mod check_command_id_no_command;
pub mod check_command_id_prefix;
pub mod check_command_name_no_command;
pub mod check_command_name_prefix;
pub mod check_global_app;
pub mod check_global_document_window;
pub mod check_global_timers;
pub mod check_instanceof_dom;
pub mod check_markdown_renderer_component;
pub mod check_navigator_usage;
pub mod check_onunload_detach;
pub mod check_plugin_onunload;
pub mod check_popover_suggest_usage;
pub mod check_promise_async_await;
pub mod check_redundant_heading;
pub mod check_regex_lookbehind;
pub mod check_settings_in_setname;
pub mod check_view_references;
pub mod node_desktop_only;
pub mod validate_license_copyright;

use crate::types::{Severity, Violation};
use std::path::Path;
use tree_sitter::{Language, Parser, Query, Tree};

/// Returns the tree-sitter TypeScript language handle.
pub(super) fn ts_language() -> Language {
    // SAFETY: `LANGUAGE_TYPESCRIPT` is a C extern returning a valid, stable `TSLanguage` pointer
    // compatible with the linked tree-sitter version. `from_raw` is the 0.22-era API; callers on
    // 0.23+ could use `Language::new`, but `from_raw` remains correct here.
    unsafe {
        Language::from_raw((tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into_raw())()
            as *const tree_sitter::ffi::TSLanguage)
    }
}

/// Parse TypeScript source, returning `(language, tree)` or `None` on failure.
/// The `Parser` is dropped after parsing — `Tree` owns its data independently.
pub(super) fn ts_parse(content: &str) -> Option<(Language, Tree)> {
    let language = ts_language();
    let mut parser = Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(content, None)?;
    Some((language, tree))
}

/// Look up a capture name in a compiled query, returning its index or `None`.
pub(super) fn capture_index(query: &Query, name: &str) -> Option<u32> {
    query
        .capture_names()
        .iter()
        .position(|n| *n == name)
        .map(|i| i as u32)
}

/// Rule data the check functions need to build violations.
/// Built from `CompiledRule` in `linter.rs` at each dispatch call site so that
/// `CompiledRule` stays private to the linter module.
pub struct CheckContext<'a> {
    pub rule_id: &'a str,
    pub rule_category: &'a str,
    pub rule_message: &'a str,
    pub rule_severity: Severity,
    pub rule_suggestion: Option<&'a str>,
    pub accuracy: &'a str,
    pub accuracy_note: Option<&'a str>,
    pub plugin_id: Option<&'a str>,
    pub plugin_name: Option<&'a str>,
}

/// Route a `use` key value to the matching built-in check function.
pub(crate) fn dispatch<'a>(
    fn_name: &str,
    path: &Path,
    content: &str,
    ctx: &CheckContext<'a>,
) -> Vec<Violation> {
    match fn_name {
        "check_node_desktop_only" => node_desktop_only::check(path, content, ctx),
        "check_callback_signature" => check_callback_signature::check(path, content, ctx),
        "check_command_id_no_command" => check_command_id_no_command::check(path, content, ctx),
        "check_command_id_prefix" => check_command_id_prefix::check(path, content, ctx),
        "check_command_name_no_command" => check_command_name_no_command::check(path, content, ctx),
        "check_command_name_prefix" => check_command_name_prefix::check(path, content, ctx),
        "check_global_app" => check_global_app::check(path, content, ctx),
        "check_global_document_window" => check_global_document_window::check(path, content, ctx),
        "check_global_timers" => check_global_timers::check(path, content, ctx),
        "check_instanceof_dom" => check_instanceof_dom::check(path, content, ctx),
        "check_markdown_renderer_component" => {
            check_markdown_renderer_component::check(path, content, ctx)
        }
        "check_navigator_usage" => check_navigator_usage::check(path, content, ctx),
        "check_onunload_detach" => check_onunload_detach::check(path, content, ctx),
        "check_plugin_onunload" => check_plugin_onunload::check(path, content, ctx),
        "check_promise_async_await" => check_promise_async_await::check(path, content, ctx),
        "check_popover_suggest_usage" => check_popover_suggest_usage::check(path, content, ctx),
        "check_redundant_heading" => check_redundant_heading::check(path, content, ctx),
        "check_regex_lookbehind" => check_regex_lookbehind::check(path, content, ctx),
        "check_settings_in_setname" => check_settings_in_setname::check(path, content, ctx),
        "check_view_references" => check_view_references::check(path, content, ctx),
        "validate_license_copyright" => validate_license_copyright::check(path, content, ctx),
        _ => {
            eprintln!("[oplint] Unknown use function: {fn_name}");
            Vec::new()
        }
    }
}

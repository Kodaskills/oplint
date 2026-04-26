use super::CheckContext;
use crate::linter::Linter;
use crate::types::Violation;
use std::path::{Path, PathBuf};

const NODE_BUILTINS: &[&str] = &[
    "fs",
    "path",
    "os",
    "crypto",
    "child_process",
    "net",
    "http",
    "https",
    "stream",
    "buffer",
    "events",
    "url",
    "util",
    "readline",
    "cluster",
    "dns",
    "tls",
    "zlib",
    "vm",
    "worker_threads",
    "perf_hooks",
    "inspector",
    "assert",
    "timers",
    "string_decoder",
    "module",
    "v8",
];

/// Flags each source file that imports a Node.js/Electron API when `"isDesktopOnly": true`
/// is absent from manifest.json.
pub fn check(
    manifest_path: &Path,
    manifest_content: &str,
    ctx: &CheckContext<'_>,
) -> Vec<Violation> {
    if Linter::detect_desktop_only(manifest_content) {
        return Vec::new();
    }

    let project_root = match manifest_path.parent() {
        Some(p) => p,
        None => return Vec::new(),
    };

    scan_ts_for_node_imports(project_root)
        .into_iter()
        .map(|(file, line)| {
            let mut v = Violation::new(
                ctx.rule_id,
                ctx.rule_category,
                ctx.rule_message,
                ctx.rule_severity.clone(),
                file,
                line,
            );
            if let Some(s) = ctx.rule_suggestion {
                v = v.with_suggestion(s);
            }
            v = v.with_accuracy(ctx.accuracy, ctx.accuracy_note);
            v
        })
        .collect()
}

fn scan_ts_for_node_imports(dir: &Path) -> Vec<(PathBuf, usize)> {
    use walkdir::WalkDir;
    let mut hits = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let path_str = path.to_string_lossy();
        if path_str.contains("node_modules") || path_str.contains(".git") {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(
            ext,
            "ts" | "tsx" | "mts" | "cts" | "js" | "jsx" | "mjs" | "cjs"
        ) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Some(line) = first_node_import_line(&content) {
                hits.push((path.to_path_buf(), line));
            }
        }
    }
    hits
}

fn first_node_import_line(content: &str) -> Option<usize> {
    for (i, line) in content.lines().enumerate() {
        for builtin in NODE_BUILTINS {
            if line.contains(&format!("from '{builtin}'"))
                || line.contains(&format!("from \"{builtin}\""))
                || line.contains(&format!("from 'node:{builtin}'"))
                || line.contains(&format!("from \"node:{builtin}\""))
                || line.contains(&format!("require('{builtin}')"))
                || line.contains(&format!("require(\"{builtin}\")"))
                || line.contains(&format!("require('node:{builtin}')"))
                || line.contains(&format!("require(\"node:{builtin}\")"))
            {
                return Some(i + 1);
            }
        }
    }
    None
}

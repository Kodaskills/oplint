use crate::config::ExcludeConfig;
use glob::Pattern;
use ignore::WalkBuilder;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn read_file(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn excluded_by_patterns(patterns: &[String], root: &Path, path: &Path) -> bool {
    if patterns.is_empty() {
        return false;
    }
    let rel = path.strip_prefix(root).unwrap_or(path);
    for raw in patterns {
        let pat_str = raw.trim_end_matches('/');
        let Ok(glob) = Pattern::new(pat_str) else {
            continue;
        };
        if glob.matches_path(rel) {
            return true;
        }
        // Check ancestors so "node_modules" excludes node_modules/foo/bar.ts
        for ancestor in rel.ancestors().skip(1) {
            if !ancestor.as_os_str().is_empty() && glob.matches_path(ancestor) {
                return true;
            }
        }
    }
    false
}

fn walk<'a>(dir: &'a Path, exclude: &'a ExcludeConfig) -> impl Iterator<Item = PathBuf> + 'a {
    WalkBuilder::new(dir)
        .follow_links(true)
        .hidden(false)
        .require_git(false)
        .git_ignore(exclude.use_gitignore)
        .git_global(exclude.use_gitignore)
        .git_exclude(exclude.use_gitignore)
        .build()
        .filter_map(|e| e.ok())
        .filter(move |e| {
            e.path().is_file() && !excluded_by_patterns(&exclude.patterns, dir, e.path())
        })
        .map(|e| e.into_path())
}

pub fn get_ts_files(dir: &Path, exclude: &ExcludeConfig) -> Vec<PathBuf> {
    walk(dir, exclude)
        .filter(|p| {
            p.extension().and_then(|e| e.to_str()).is_some_and(|e| {
                matches!(
                    e,
                    "ts" | "tsx" | "mts" | "cts" | "js" | "jsx" | "mjs" | "cjs"
                )
            })
        })
        .collect()
}

pub fn get_json_files(dir: &Path, exclude: &ExcludeConfig) -> Vec<PathBuf> {
    walk(dir, exclude)
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect()
}

pub fn find_manifest(dir: &Path) -> Option<PathBuf> {
    let manifest_path = dir.join("manifest.json");
    if manifest_path.exists() {
        return Some(manifest_path);
    }

    for entry in WalkDir::new(dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name() {
                if name.to_str() == Some("manifest.json") {
                    return Some(path.to_path_buf());
                }
            }
        }
    }

    None
}

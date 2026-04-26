use oplint::file_helper as file_utils;
use oplint::ExcludeConfig;
use std::fs;
use tempfile::tempdir;

fn no_exclude() -> ExcludeConfig {
    ExcludeConfig {
        use_gitignore: false,
        patterns: vec![],
    }
}

#[test]
fn test_read_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "hello world").unwrap();

    assert_eq!(
        file_utils::read_file(&file_path),
        Some("hello world".to_string())
    );
    assert_eq!(file_utils::read_file(&dir.path().join("missing.txt")), None);
}

#[test]
fn test_get_ts_files() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("file1.ts"), "").unwrap();
    fs::write(dir.path().join("file2.tsx"), "").unwrap();
    fs::write(dir.path().join("file3.js"), "").unwrap();
    fs::write(dir.path().join("file4.jsx"), "").unwrap();
    fs::write(dir.path().join("other.txt"), "").unwrap();

    let exclude = ExcludeConfig {
        use_gitignore: false,
        patterns: vec!["node_modules".to_string()],
    };
    let node_modules = dir.path().join("node_modules");
    fs::create_dir(&node_modules).unwrap();
    fs::write(node_modules.join("ignored.ts"), "").unwrap();

    let files = file_utils::get_ts_files(dir.path(), &exclude);
    assert_eq!(files.len(), 4);

    let filenames: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();
    assert!(filenames.contains(&"file1.ts"));
    assert!(filenames.contains(&"file2.tsx"));
    assert!(filenames.contains(&"file3.js"));
    assert!(filenames.contains(&"file4.jsx"));
    assert!(!filenames.contains(&"ignored.ts"));
}

#[test]
fn test_get_ts_files_no_exclude_includes_all() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("main.ts"), "").unwrap();
    let node_modules = dir.path().join("node_modules");
    fs::create_dir(&node_modules).unwrap();
    fs::write(node_modules.join("dep.ts"), "").unwrap();

    let files = file_utils::get_ts_files(dir.path(), &no_exclude());
    let filenames: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();
    assert!(filenames.contains(&"main.ts"));
    assert!(filenames.contains(&"dep.ts"));
}

#[test]
fn test_exclude_glob_pattern() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("main.ts"), "").unwrap();

    let dist = dir.path().join("dist");
    fs::create_dir(&dist).unwrap();
    fs::write(dist.join("bundle.js"), "").unwrap();

    let coverage = dir.path().join("coverage");
    fs::create_dir(&coverage).unwrap();
    fs::write(coverage.join("report.ts"), "").unwrap();

    let exclude = ExcludeConfig {
        use_gitignore: false,
        patterns: vec!["dist/".to_string(), "coverage/".to_string()],
    };

    let files = file_utils::get_ts_files(dir.path(), &exclude);
    let filenames: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();
    assert!(filenames.contains(&"main.ts"));
    assert!(!filenames.contains(&"bundle.js"));
    assert!(!filenames.contains(&"report.ts"));
}

#[test]
fn test_gitignore_respected() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("main.ts"), "").unwrap();

    let node_modules = dir.path().join("node_modules");
    fs::create_dir(&node_modules).unwrap();
    fs::write(node_modules.join("dep.ts"), "").unwrap();

    fs::write(dir.path().join(".gitignore"), "node_modules\n").unwrap();

    let exclude = ExcludeConfig {
        use_gitignore: true,
        patterns: vec![],
    };

    let files = file_utils::get_ts_files(dir.path(), &exclude);
    let filenames: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();
    assert!(filenames.contains(&"main.ts"));
    assert!(!filenames.contains(&"dep.ts"));
}

#[test]
fn test_gitignore_disabled_includes_ignored_dirs() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("main.ts"), "").unwrap();

    let node_modules = dir.path().join("node_modules");
    fs::create_dir(&node_modules).unwrap();
    fs::write(node_modules.join("dep.ts"), "").unwrap();

    fs::write(dir.path().join(".gitignore"), "node_modules\n").unwrap();

    let exclude = ExcludeConfig {
        use_gitignore: false,
        patterns: vec![],
    };

    let files = file_utils::get_ts_files(dir.path(), &exclude);
    let filenames: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();
    assert!(filenames.contains(&"dep.ts"));
}

#[test]
fn test_get_json_files() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("data.json"), "").unwrap();
    fs::write(dir.path().join("other.txt"), "").unwrap();

    let files = file_utils::get_json_files(dir.path(), &no_exclude());
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_name().unwrap().to_str().unwrap(), "data.json");
}

#[test]
fn test_find_manifest() {
    let dir = tempdir().unwrap();

    let manifest_path = dir.path().join("manifest.json");
    fs::write(&manifest_path, "{}").unwrap();
    assert_eq!(
        file_utils::find_manifest(dir.path()),
        Some(manifest_path.clone())
    );

    fs::remove_file(&manifest_path).unwrap();
    let sub = dir.path().join("plugin");
    fs::create_dir(&sub).unwrap();
    let nested_manifest = sub.join("manifest.json");
    fs::write(&nested_manifest, "{}").unwrap();
    assert_eq!(file_utils::find_manifest(dir.path()), Some(nested_manifest));
}

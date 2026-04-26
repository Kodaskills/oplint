use oplint::Linter;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use tempfile::tempdir;

static LINTER: OnceLock<Linter> = OnceLock::new();

fn get_linter() -> &'static Linter {
    LINTER.get_or_init(|| Linter::new_with_config(None))
}

fn run_rule_test(
    rule_id: &str,
    content: &str,
    is_manifest: bool,
    expected_violations: usize,
    description: &str,
) {
    let linter = get_linter();
    let path = if is_manifest {
        Path::new("manifest.json")
    } else {
        Path::new("test.ts")
    };

    let violations = if is_manifest {
        linter.lint_manifest(path, content)
    } else {
        linter.lint_file(path, content)
    };

    let count = violations.iter().filter(|v| v.rule_id == rule_id).count();
    assert_eq!(
        count, expected_violations,
        "Rule {} failed: {}. Content: {}. Expected {} violations, found {}.",
        rule_id, description, content, expected_violations, count
    );
}

macro_rules! rule_test {
    ($name:ident, $id:expr, $content:expr, $is_manifest:expr, $expected:expr, $desc:expr) => {
        #[test]
        fn $name() {
            run_rule_test($id, $content, $is_manifest, $expected, $desc);
        }
    };
}

// --- SECURITY RULES ---
rule_test!(
    test_sec001_bad,
    "SEC001",
    "el.innerHTML = 'foo';",
    false,
    1,
    "Bad: innerHTML usage"
);
rule_test!(
    test_sec001_good,
    "SEC001",
    "el.textContent = 'foo';",
    false,
    0,
    "Good: textContent instead of innerHTML"
);
rule_test!(
    test_sec002_bad,
    "SEC002",
    "el.outerHTML = 'foo';",
    false,
    1,
    "Bad: outerHTML usage"
);
rule_test!(
    test_sec003_bad,
    "SEC003",
    "el.insertAdjacentHTML('afterbegin', 'foo');",
    false,
    1,
    "Bad: insertAdjacentHTML usage"
);
rule_test!(
    test_sec004_bad,
    "SEC004",
    "window.app.vault.getFiles();",
    false,
    1,
    "Bad: window.app usage"
);
rule_test!(
    test_sec004_good,
    "SEC004",
    "this.app.vault.getFiles();",
    false,
    0,
    "Good: this.app usage"
);

// --- RESOURCE RULES ---
rule_test!(
    test_res001_bad,
    "RES001",
    "class MyPlugin extends Plugin { onload() {} }",
    false,
    1,
    "Bad: missing onunload"
);
rule_test!(
    test_res001_good,
    "RES001",
    "class MyPlugin extends Plugin { onunload() {} }",
    false,
    0,
    "Good: has onunload"
);
rule_test!(
    test_res002_bad,
    "RES002",
    "class MyPlugin extends Plugin { onunload() { this.app.workspace.getLeaf().detach(); } }",
    false,
    1,
    "Bad: detaching leaf in onunload"
);

// --- VAULT RULES ---
rule_test!(
    test_vault001_bad,
    "VAULT001",
    "this.app.vault.modify(file, 'data');",
    false,
    1,
    "Bad: using vault.modify instead of process"
);
rule_test!(
    test_vault001_good,
    "VAULT001",
    "this.app.vault.process(file, (d) => 'data');",
    false,
    0,
    "Good: using vault.process"
);
rule_test!(
    test_vault002_bad,
    "VAULT002",
    "this.app.vault.getFiles();",
    false,
    1,
    "Bad: using vault.getFiles for search"
);
rule_test!(
    test_vault003_match,
    "VAULT003",
    "normalizePath('foo');",
    false,
    1,
    "Confirms: flags if normalizePath is used"
);

// --- WORKSPACE RULES ---
rule_test!(
    test_work001_bad,
    "WORK001",
    "this.app.workspace.activeLeaf;",
    false,
    1,
    "Bad: accessing activeLeaf directly"
);

// --- COMMAND RULES ---
rule_test!(
    test_cmd001_bad,
    "CMD001",
    "this.addCommand({ id: 'foo', hotkeys: [] });",
    false,
    1,
    "Bad: setting default hotkeys"
);
rule_test!(
    test_cmd002_bad,
    "CMD002",
    r#"this.addCommand({ id: "foo", editorCallback: () => {} });"#,
    false,
    1,
    "Bad: editorCallback without editor param"
);
rule_test!(
    test_cmd002_good,
    "CMD002",
    r#"this.addCommand({ id: "foo", editorCallback: (editor) => {} });"#,
    false,
    0,
    "Good: editorCallback with editor param"
);
rule_test!(
    test_cmd005_bad,
    "CMD005",
    "this.addCommand({ id: 'foo', name: 'My Command' });",
    false,
    1,
    "Bad: including 'command' in name"
);

// --- UI RULES ---
rule_test!(
    test_ui002_bad,
    "UI002",
    "new Setting(containerEl).setName('Plugin settings');",
    false,
    1,
    "Bad: 'settings' in setting name"
);
rule_test!(
    test_ui003_baseline,
    "UI003",
    "new Setting(containerEl).setName('My Cool Feature');",
    false,
    0,
    "Baseline: Currently failing to match Title Case"
);
rule_test!(
    test_ui005_bad,
    "UI005",
    "containerEl.createEl('h3');",
    false,
    1,
    "Bad: manual H3 heading"
);

// --- STYLING RULES ---
rule_test!(
    test_style001_bad,
    "STYLE001",
    "el.style.color = 'red';",
    false,
    1,
    "Bad: hardcoded style"
);

// --- TYPESCRIPT RULES ---
rule_test!(
    test_ts001_bad,
    "TS001",
    "var x = 1;",
    false,
    1,
    "Bad: using var"
);
rule_test!(
    test_ts001_good,
    "TS001",
    "let x = 1;",
    false,
    0,
    "Good: using let"
);
rule_test!(
    test_ts002_bad_unnecessary_wrapper,
    "TS002",
    "const p = new Promise((resolve) => { const data = getData(); resolve(data); });",
    false,
    1,
    "Bad: unnecessary Promise wrapper"
);
rule_test!(
    test_ts002_promisify_set_timeout_good,
    "TS002",
    "return new Promise((resolve) => setTimeout(resolve, ms));",
    false,
    0,
    "Good: promisify pattern with setTimeout"
);
rule_test!(
    test_ts002_promisify_callback_good,
    "TS002",
    "return new Promise((resolve, reject) => fs.readFile(path, (err, data) => { if (err) reject(err); else resolve(data); }));",
    false,
    0,
    "Good: promisify pattern with callback"
);
rule_test!(
    test_ts002_promisify_then_catch_good,
    "TS002",
    "new Promise((resolve, reject) => { someAsyncThing().then(resolve).catch(reject); });",
    false,
    0,
    "Good: promisify pattern with then/catch"
);

// --- GENERAL RULES ---
rule_test!(
    test_gen001_bad,
    "GEN001",
    "console.log('hi');",
    false,
    1,
    "Bad: console.log usage"
);
rule_test!(
    test_gen002_bad,
    "GEN002",
    "class MyPlugin extends Plugin {}",
    false,
    1,
    "Bad: placeholder class name"
);
rule_test!(
    test_gen003_bad,
    "GEN003",
    "app.vault.getFiles();",
    false,
    1,
    "Bad: bare global app"
);
rule_test!(
    test_gen003_ok_function_param,
    "GEN003",
    "function helper(app: App) { app.vault.getFiles(); }",
    false,
    0,
    "Ok: app is function parameter"
);
rule_test!(
    test_gen003_ok_arrow_param,
    "GEN003",
    "const f = (app: App) => app.vault.getFiles();",
    false,
    0,
    "Ok: app is arrow function parameter"
);
rule_test!(
    test_gen003_ok_method_param,
    "GEN003",
    "class X { doStuff(app: App) { app.vault.getFiles(); } }",
    false,
    0,
    "Ok: app is method parameter"
);
rule_test!(
    test_gen003_ok_constructor_param,
    "GEN003",
    "class X { constructor(private app: App) { app.vault.getFiles(); } }",
    false,
    0,
    "Ok: app is constructor parameter"
);
rule_test!(
    test_gen003_bad_unrelated_method,
    "GEN003",
    "class X { constructor(private app: App) {} onOpen() { app.workspace.getLeaf(); } }",
    false,
    1,
    "Bad: app used in method where it is not a parameter"
);
rule_test!(
    test_gen003_ok_derived_from_param,
    "GEN003",
    "async function openLeaf(ea: ExcalidrawAutomate) { const app = ea.plugin.app; app.vault.getFiles(); }",
    false,
    0,
    "Ok: app derived from function parameter via member chain"
);
rule_test!(
    test_gen003_ok_derived_deep_chain,
    "GEN003",
    "function foo(ctx: Context) { const app = ctx.state.manager.app; app.workspace.getLeaf(); }",
    false,
    0,
    "Ok: app derived from param via deep member chain"
);
rule_test!(
    test_gen003_ok_derived_typed_param,
    "GEN003",
    "async function openLeaf(ea: ExcalidrawAutomate, settings: Settings, leaf: WorkspaceLeaf) { const app = ea.plugin.app; app.vault.getAbstractFileByPath(settings.path); }",
    false,
    0,
    "Ok: exact excalibrain pattern - app from typed param"
);
rule_test!(
    test_gen003_bad_derived_from_nonparam,
    "GEN003",
    "function foo() { const app = globalObj.plugin.app; app.vault.getFiles(); }",
    false,
    1,
    "Bad: app derived from non-param global"
);

// --- MOBILE RULES ---
rule_test!(
    test_mob001_bad,
    "MOB001",
    "/(?<=a)b/;",
    false,
    1,
    "Bad: regex lookbehind"
);
rule_test!(
    test_mob002_bad,
    "MOB002",
    "setTimeout(() => {}, 100);",
    false,
    1,
    "Bad: global setTimeout"
);
rule_test!(
    test_mob003_bad,
    "MOB003",
    "window.innerWidth;",
    false,
    1,
    "Bad: window usage"
);
rule_test!(
    test_mob004_bad,
    "MOB004",
    "navigator.userAgent;",
    false,
    1,
    "Bad: navigator usage"
);

// --- API RULES ---
rule_test!(
    test_api001_bad,
    "API001",
    "this.app.vault.adapter.read('f');",
    false,
    1,
    "Bad: vault.adapter usage"
);
rule_test!(
    test_api002_bad,
    "API002",
    "this.app.vault.trash(f, true);",
    false,
    1,
    "Bad: vault.trash usage"
);
rule_test!(
    test_api003_bad,
    "API003",
    "f as TFile;",
    false,
    1,
    "Bad: casting as TFile"
);
rule_test!(
    test_api007_bad,
    "API007",
    "localStorage.getItem('language');",
    false,
    1,
    "Bad: localStorage for language"
);

// --- MANIFEST RULES ---
rule_test!(
    test_man001_bad,
    "MAN001",
    r#"{}"#,
    true,
    1,
    "Bad: missing id in manifest"
);
rule_test!(
    test_man001_good,
    "MAN001",
    r#"{"id": "test"}"#,
    true,
    0,
    "Good: has id in manifest"
);
rule_test!(
    test_man007_bad,
    "MAN007",
    r#"{"id": "MyPlugin"}"#,
    true,
    1,
    "Bad: invalid ID format"
);
rule_test!(
    test_man010_baseline,
    "MAN010",
    r#"{"description": "No period"}"#,
    true,
    0,
    "Baseline: Currently failing to match missing period"
);
rule_test!(
    test_man011_bad,
    "MAN011",
    r#"{"id": "sample-plugin"}"#,
    true,
    1,
    "Bad: sample plugin ID"
);

// --- MAN008: Node.js API Detection ---

#[test]
fn test_man008_node_detection_bad() {
    let dir = tempdir().unwrap();
    let manifest_path = dir.path().join("manifest.json");
    let src_path = dir.path().join("main.ts");

    fs::write(&manifest_path, r#"{"id": "test-plugin"}"#).unwrap();
    fs::write(&src_path, r#"import { readFileSync } from 'fs';"#).unwrap();

    let linter = get_linter();
    let violations = linter.lint_manifest(&manifest_path, r#"{"id": "test-plugin"}"#);

    let count = violations.iter().filter(|v| v.rule_id == "MAN008").count();
    assert_eq!(
        count, 1,
        "Should flag MAN008 if source imports node builtin and isDesktopOnly is missing"
    );
}

#[test]
fn test_man008_node_detection_good_flag() {
    let dir = tempdir().unwrap();
    let manifest_path = dir.path().join("manifest.json");

    // isDesktopOnly: true is present
    let manifest_content = r#"{"id": "test-plugin", "isDesktopOnly": true}"#;
    fs::write(&manifest_path, manifest_content).unwrap();

    let linter = get_linter();
    let violations = linter.lint_manifest(&manifest_path, manifest_content);

    let count = violations.iter().filter(|v| v.rule_id == "MAN008").count();
    assert_eq!(count, 0, "Should NOT flag MAN008 if isDesktopOnly is true");
}

// --- CMD003 tests ---

fn linter_with_plugin(id: &str) -> Linter {
    let mut l = Linter::new_with_config(None);
    l.set_plugin_id(id);
    l
}

#[test]
fn test_cmd003_no_violation_unrelated_prefix() {
    let l = linter_with_plugin("po-editor");
    let src = r#"this.addCommand({ id: "po-unmark-fuzzy", name: "x", callback: () => {} });"#;
    let v: Vec<_> = l
        .lint_file(Path::new("test.ts"), src)
        .into_iter()
        .filter(|v| v.rule_id == "CMD003")
        .collect();
    assert_eq!(
        v.len(),
        0,
        "po-unmark-fuzzy must NOT be flagged: no colon-separated prefix"
    );
}

#[test]
fn test_cmd003_violation_full_prefix() {
    let l = linter_with_plugin("po-editor");
    let src =
        r#"this.addCommand({ id: "po-editor:unmark-fuzzy", name: "x", callback: () => {} });"#;
    let v: Vec<_> = l
        .lint_file(Path::new("test.ts"), src)
        .into_iter()
        .filter(|v| v.rule_id == "CMD003")
        .collect();
    assert_eq!(v.len(), 1, "po-editor:unmark-fuzzy MUST be flagged");
}

#[test]
fn test_cmd003_no_violation_no_prefix() {
    let l = linter_with_plugin("po-editor");
    let src = r#"this.addCommand({ id: "unmark-fuzzy", name: "x", callback: () => {} });"#;
    let v: Vec<_> = l
        .lint_file(Path::new("test.ts"), src)
        .into_iter()
        .filter(|v| v.rule_id == "CMD003")
        .collect();
    assert_eq!(v.len(), 0, "unmark-fuzzy must NOT be flagged");
}

#[test]
fn test_man008_node_detection_import_bad() {
    let dir = tempdir().unwrap();
    let manifest_path = dir.path().join("manifest.json");
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    let ts_file = src_dir.join("main.ts");

    fs::write(&manifest_path, r#"{"id": "test-plugin"}"#).unwrap();
    fs::write(&ts_file, r#"import { readFileSync } from 'fs';"#).unwrap();

    let linter = get_linter();
    let violations = linter.lint_manifest(&manifest_path, r#"{"id": "test-plugin"}"#);

    let count = violations.iter().filter(|v| v.rule_id == "MAN008").count();
    assert_eq!(
        count, 1,
        "Should flag MAN008 if Node.js imports are found in source files"
    );
}

// --- UI002 tests ---

rule_test!(
    test_ui002_settings_in_setname_bad,
    "UI002",
    r#"new Setting(el).setName("Advanced settings");"#,
    false,
    1,
    "setName with 'settings' must be flagged"
);
rule_test!(
    test_ui002_settings_uppercase_bad,
    "UI002",
    r#"new Setting(el).setName("Plugin Settings");"#,
    false,
    1,
    "setName with 'Settings' (capital) must be flagged"
);
rule_test!(
    test_ui002_no_settings_good,
    "UI002",
    r#"new Setting(el).setName("Advanced");"#,
    false,
    0,
    "setName without 'settings' must NOT be flagged"
);
rule_test!(
    test_ui002_other_method_good,
    "UI002",
    r#"new Setting(el).setDesc("Advanced settings description");"#,
    false,
    0,
    "setDesc with 'settings' must NOT be flagged — only setName is checked"
);

// --- CMD002 tests ---

rule_test!(
    test_cmd002_editor_callback_no_editor_param_bad,
    "CMD002",
    r#"this.addCommand({ id: "x", editorCallback: (checking) => {} });"#,
    false,
    1,
    "editorCallback without editor param must be flagged"
);
rule_test!(
    test_cmd002_editor_callback_with_editor_param_good,
    "CMD002",
    r#"this.addCommand({ id: "x", editorCallback: (editor, view) => {} });"#,
    false,
    0,
    "editorCallback with editor param must NOT be flagged"
);
rule_test!(
    test_cmd002_callback_with_editor_param_bad,
    "CMD002",
    r#"this.addCommand({ id: "x", callback: (editor) => {} });"#,
    false,
    1,
    "callback with editor param must be flagged"
);
rule_test!(
    test_cmd002_callback_no_editor_param_good,
    "CMD002",
    r#"this.addCommand({ id: "x", callback: () => {} });"#,
    false,
    0,
    "callback without editor param must NOT be flagged"
);
rule_test!(
    test_cmd002_reference_value_skipped,
    "CMD002",
    r#"this.addCommand({ id: "x", editorCallback: this.handleEditor });"#,
    false,
    0,
    "identifier reference (not inline fn) must be skipped"
);
rule_test!(
    test_cmd002_check_callback_good,
    "CMD002",
    r#"this.addCommand({ id: "x", checkCallback: (checking) => {} });"#,
    false,
    0,
    "checkCallback without editor param must NOT be flagged"
);

// --- RES001 tests ---

#[test]
fn test_res001_missing_onunload_bad() {
    let src = r#"
class MyPlugin extends Plugin {
  onload() {}
}
"#;
    run_rule_test(
        "RES001",
        src,
        false,
        1,
        "Plugin class without onunload must be flagged",
    );
}

#[test]
fn test_res001_has_onunload_good() {
    let src = r#"
class MyPlugin extends Plugin {
  onload() {}
  onunload() {}
}
"#;
    run_rule_test(
        "RES001",
        src,
        false,
        0,
        "Plugin class with onunload must NOT be flagged",
    );
}

#[test]
fn test_res001_non_plugin_class_good() {
    let src = r#"
class MyView extends ItemView {
  onload() {}
}
"#;
    run_rule_test(
        "RES001",
        src,
        false,
        0,
        "Non-plugin class must NOT be flagged",
    );
}

#[test]
fn test_res001_no_class_good() {
    let src = r#"
function helper() { return 42; }
"#;
    run_rule_test("RES001", src, false, 0, "No class — must NOT flag");
}

// --- RES002 tests ---

#[test]
fn test_res002_detach_in_onunload_bad() {
    let src = r#"
class MyPlugin extends Plugin {
  onunload() {
    this.leaf.detach();
  }
}
"#;
    run_rule_test(
        "RES002",
        src,
        false,
        1,
        "detach() inside onunload() must be flagged",
    );
}

#[test]
fn test_res002_detach_outside_onunload_good() {
    let src = r#"
class MyPlugin extends Plugin {
  someMethod() {
    this.leaf.detach();
  }
}
"#;
    run_rule_test(
        "RES002",
        src,
        false,
        0,
        "detach() outside onunload() must NOT be flagged",
    );
}

#[test]
fn test_res002_no_onunload_good() {
    let src = r#"
class MyPlugin extends Plugin {
  onload() {
    this.leaf.detach();
  }
}
"#;
    run_rule_test(
        "RES002",
        src,
        false,
        0,
        "no onunload method — must NOT flag",
    );
}

#[test]
fn test_res002_multiple_detach_in_onunload() {
    let src = r#"
class MyPlugin extends Plugin {
  onunload() {
    this.leafA.detach();
    this.leafB.detach();
  }
}
"#;
    run_rule_test(
        "RES002",
        src,
        false,
        2,
        "two detach() in onunload must produce 2 violations",
    );
}

// --- WORK002 tests ---

#[test]
fn test_work002_view_ref_in_register_bad() {
    let src = r#"
class MyPlugin extends Plugin {
  onload() {
    this.registerViewType(VIEW_TYPE, () => this.view = new MyView(this.app));
  }
}
"#;
    run_rule_test(
        "WORK002",
        src,
        false,
        1,
        "storing view ref in registerViewType must be flagged",
    );
}

#[test]
fn test_work002_no_assignment_good() {
    let src = r#"
class MyPlugin extends Plugin {
  onload() {
    this.registerViewType(VIEW_TYPE, () => new MyView(this.app));
  }
}
"#;
    run_rule_test(
        "WORK002",
        src,
        false,
        0,
        "no this.x = new View() — must NOT flag",
    );
}

#[test]
fn test_work002_no_register_good() {
    let src = r#"
class MyPlugin extends Plugin {
  onload() {
    this.view = new MyView(this.app);
  }
}
"#;
    run_rule_test(
        "WORK002",
        src,
        false,
        0,
        "assignment outside registerViewType — must NOT flag",
    );
}

// --- API004 tests ---

#[test]
fn test_api004_html_element_bad() {
    let src = r#"
if (el instanceof HTMLElement) {}
"#;
    run_rule_test(
        "API004",
        src,
        false,
        1,
        "instanceof HTMLElement must be flagged",
    );
}

#[test]
fn test_api004_mouse_event_bad() {
    let src = r#"
if (e instanceof MouseEvent) {}
"#;
    run_rule_test(
        "API004",
        src,
        false,
        1,
        "instanceof MouseEvent must be flagged",
    );
}

#[test]
fn test_api004_multiple_bad() {
    let src = r#"
if (el instanceof HTMLDivElement) {}
if (e instanceof KeyboardEvent) {}
"#;
    run_rule_test(
        "API004",
        src,
        false,
        2,
        "two known DOM types must produce 2 violations",
    );
}

#[test]
fn test_api004_unknown_class_good() {
    let src = r#"
if (x instanceof MyCustomClass) {}
"#;
    run_rule_test(
        "API004",
        src,
        false,
        0,
        "unknown class not in whitelist must NOT flag",
    );
}

#[test]
fn test_api004_tfile_instanceof_good() {
    let src = r#"
if (f instanceof TFile) {}
"#;
    run_rule_test(
        "API004",
        src,
        false,
        0,
        "TFile is not in DOM whitelist — must NOT flag",
    );
}

#[test]
fn test_api005_this_as_component_bad() {
    let src = r#"
await MarkdownRenderer.render(content, this, sourcePath);
"#;
    run_rule_test(
        "API005",
        src,
        false,
        1,
        "MarkdownRenderer.render with this must be flagged",
    );
}

#[test]
fn test_api005_dedicated_el_good() {
    let src = r#"
const el = this.containerEl.createDiv();
await MarkdownRenderer.render(content, el, sourcePath);
"#;
    run_rule_test(
        "API005",
        src,
        false,
        0,
        "MarkdownRenderer.render with dedicated el must NOT flag",
    );
}

#[test]
fn test_api005_other_method_good() {
    let src = r#"
await MarkdownRenderer.renderMarkdown(content, this, sourcePath, this);
"#;
    run_rule_test(
        "API005",
        src,
        false,
        0,
        "renderMarkdown (not render) must NOT flag",
    );
}

#[test]
fn test_api006_extends_popover_suggest_bad() {
    let src = r#"
class MySuggest extends PopoverSuggest<string> {}
"#;
    run_rule_test(
        "API006",
        src,
        false,
        1,
        "extends PopoverSuggest must be flagged",
    );
}

#[test]
fn test_api006_extends_abstract_input_suggest_good() {
    let src = r#"
class MySuggest extends AbstractInputSuggest<string> {}
"#;
    run_rule_test(
        "API006",
        src,
        false,
        0,
        "extends AbstractInputSuggest must NOT flag",
    );
}

#[test]
fn test_api006_extends_other_class_good() {
    let src = r#"
class MyPlugin extends Plugin {}
"#;
    run_rule_test("API006", src, false, 0, "extends Plugin must NOT flag");
}

#[test]
fn test_ui004_general_heading_bad() {
    let src = r#"section.setName("General");"#;
    run_rule_test("UI004", src, false, 1, "setName('General') must be flagged");
}

#[test]
fn test_ui004_settings_heading_bad() {
    let src = r#"section.setName("Settings");"#;
    run_rule_test(
        "UI004",
        src,
        false,
        1,
        "setName('Settings') must be flagged",
    );
}

#[test]
fn test_ui004_options_heading_bad() {
    let src = r#"section.setName("Options");"#;
    run_rule_test("UI004", src, false, 1, "setName('Options') must be flagged");
}

#[test]
fn test_ui004_custom_heading_good() {
    let src = r#"section.setName("Advanced");"#;
    run_rule_test("UI004", src, false, 0, "custom heading must NOT flag");
}

#[test]
fn test_cmd004_id_contains_command_bad() {
    let src = r#"this.addCommand({ id: "my-command-open", name: "Open" });"#;
    run_rule_test(
        "CMD004",
        src,
        false,
        1,
        "id containing 'command' must be flagged",
    );
}

#[test]
fn test_cmd004_id_contains_command_uppercase_bad() {
    let src = r#"this.addCommand({ id: "myCommand", name: "Do thing" });"#;
    run_rule_test(
        "CMD004",
        src,
        false,
        1,
        "camelCase 'Command' in id must be flagged",
    );
}

#[test]
fn test_cmd004_clean_id_good() {
    let src = r#"this.addCommand({ id: "open-note", name: "Open note" });"#;
    run_rule_test(
        "CMD004",
        src,
        false,
        0,
        "id without 'command' must NOT flag",
    );
}

#[test]
fn test_cmd005_name_contains_command_bad() {
    let src = r#"this.addCommand({ id: "open-note", name: "Open command" });"#;
    run_rule_test(
        "CMD005",
        src,
        false,
        1,
        "name containing 'command' must be flagged",
    );
}

#[test]
fn test_cmd005_name_contains_command_uppercase_bad() {
    let src = r#"this.addCommand({ id: "open-note", name: "Run Command" });"#;
    run_rule_test(
        "CMD005",
        src,
        false,
        1,
        "uppercase 'Command' in name must be flagged",
    );
}

#[test]
fn test_cmd005_clean_name_good() {
    let src = r#"this.addCommand({ id: "open-note", name: "Open note" });"#;
    run_rule_test(
        "CMD005",
        src,
        false,
        0,
        "name without 'command' must NOT flag",
    );
}

#[test]
fn test_mob001_regex_literal_lookbehind_bad() {
    let src = r#"const re = /(?<=foo)bar/;"#;
    run_rule_test(
        "MOB001",
        src,
        false,
        1,
        "regex literal with lookbehind must be flagged",
    );
}

#[test]
fn test_mob001_new_regexp_lookbehind_bad() {
    let src = r#"const re = new RegExp("(?<!foo)bar");"#;
    run_rule_test(
        "MOB001",
        src,
        false,
        1,
        "new RegExp() with lookbehind must be flagged",
    );
}

#[test]
fn test_mob001_no_lookbehind_good() {
    let src = r#"const re = /foobar/;"#;
    run_rule_test(
        "MOB001",
        src,
        false,
        0,
        "regex without lookbehind must NOT flag",
    );
}

#[test]
fn test_mob002_global_settimeout_bad() {
    let src = r#"setTimeout(() => {}, 100);"#;
    run_rule_test("MOB002", src, false, 1, "bare setTimeout must be flagged");
}

#[test]
fn test_mob002_active_window_settimeout_good() {
    let src = r#"activeWindow.setTimeout(() => {}, 100);"#;
    run_rule_test(
        "MOB002",
        src,
        false,
        0,
        "activeWindow.setTimeout must NOT flag",
    );
}

#[test]
fn test_mob003_document_access_bad() {
    let src = r#"const el = document.createElement("div");"#;
    run_rule_test(
        "MOB003",
        src,
        false,
        1,
        "bare document access must be flagged",
    );
}

#[test]
fn test_mob003_window_access_bad() {
    let src = r#"const w = window.innerWidth;"#;
    run_rule_test(
        "MOB003",
        src,
        false,
        1,
        "bare window access must be flagged",
    );
}

#[test]
fn test_mob003_active_document_good() {
    let src = r#"const el = activeDocument.createElement("div");"#;
    run_rule_test(
        "MOB003",
        src,
        false,
        0,
        "activeDocument access must NOT flag",
    );
}

#[test]
fn test_mob004_navigator_bad() {
    let src = r#"const ua = navigator.userAgent;"#;
    run_rule_test("MOB004", src, false, 1, "navigator access must be flagged");
}

#[test]
fn test_mob004_platform_good() {
    let src = r#"const isMobile = Platform.isMobile;"#;
    run_rule_test("MOB004", src, false, 0, "Platform access must NOT flag");
}

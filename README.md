<div align="center">

<img src="docs/logo.svg" alt="oplint logo" width="128" height="128" />

# OPLint — Obsidian Plugin Linter

### Static analysis CLI for **Obsidian plugin** compliance — security, manifest validation, mobile compatibility, API best practices, and UI guidelines.

[![Crates.io](https://img.shields.io/crates/v/oplint?style=for-the-badge&logo=rust&color=orange)](https://crates.io/crates/oplint)
[![License](https://img.shields.io/badge/License-MIT-green.svg?style=for-the-badge)](LICENSE)
[![Last Commit](https://img.shields.io/github/last-commit/kodaskills/oplint/main?style=for-the-badge)](https://github.com/kodaskills/oplint/commits/main)

### Built with:
[![Rust](https://img.shields.io/badge/Rust-2021-CE422B?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![tree-sitter](https://img.shields.io/badge/tree--sitter-0.22-5B8FB9?style=for-the-badge)](https://tree-sitter.github.io)

</div>

---

## ✨ Features

| Area | What OPLint checks |
|------|--------------------|
| **Security** | `innerHTML`, `outerHTML`, `insertAdjacentHTML`, global `app` access |
| **Manifest** | Required fields, valid plugin ID, description length, `isDesktopOnly` flag |
| **Mobile** | Lookbehind regex, global timers, `document`/`window` globals, `navigator` API |
| **Resources** | `onunload` implementation, leaf detachment, `MarkdownRenderer` component misuse |
| **Vault API** | `vault.modify` vs `vault.process`, adapter bypass, `FileManager.trashFile` |
| **Workspace** | `activeLeaf` usage, stored view references |
| **Commands** | Default hotkeys, callback types, ID/name prefixes |
| **UI** | `setHeading`, sentence case, redundant headings, hardcoded styles |
| **TypeScript** | `var` usage, raw `Promise` chains, `as TFile`/`as TFolder` casts |
| **General** | `console.log`, placeholder class names, bare `app` global |

**50+ rules**, tree-sitter AST-based, accuracy-tagged (`exact` / `approximate`).

---

## 🚀 Installation

**One-liner** (recommended — macOS & Linux, no Rust needed):

```bash
curl -fsSL https://raw.githubusercontent.com/kodaskills/oplint/main/install.sh | sh
```

**Via cargo** (requires Rust 1.74+):

```bash
cargo install oplint
```

**Pre-built binary** — download manually for your platform from [GitHub Releases](https://github.com/kodaskills/oplint/releases/latest) (Linux, macOS, Windows).

Or build from source:

```bash
git clone https://github.com/kodaskills/oplint
cd oplint
cargo build --release
./target/release/oplint --help
```

---

## ⚡ Quick Start

```bash
# Lint a plugin directory (table format by default)
oplint lint /path/to/my-obsidian-plugin

# HTML report — open in browser
oplint lint /path/to/my-obsidian-plugin -f html > report.html

# JSON output — pipe into CI scripts
oplint lint /path/to/my-obsidian-plugin -f json | jq '.summary'

# List all available rules
oplint rules

# Generate a config file
oplint init
```

---

## 📋 Output Formats

| Format | Flag | Best for |
|--------|------|----------|
| Table | `table` (default) | Terminal — box-drawing table with color |
| Terminal | `terminal` | CI — compact, one line per violation |
| HTML | `html` | Reports — interactive, filterable, expandable |
| Markdown | `markdown` / `md` | GitHub PRs, wikis |
| JSON | `json` | CI/CD pipelines, scripting |
| YAML | `yaml` | Config tooling |
| TOML | `toml` | Rust tooling integration |

All formats include a **compliance score** (0–100), **grade** (A–F), and **performance stats** (total / avg / min / max per file).

---

## 📏 Rules

Rules are tagged with an accuracy level indicating reliability:

| Tag | Meaning |
|-----|---------|
| `exact` | Fully reliable — semantic analysis, no false positives/negatives |
| `approximate` | Best-effort — may miss edge cases or have false positives |

Approximate rules display a note in all report formats explaining the limitation.

For the full rule reference (IDs, descriptions, severities, accuracy levels), see **[oplint.kodaskills.co/#rules](https://oplint.kodaskills.co/#rules)**.

> **Rules are battle-tested against a wide range of real-world plugins**, but static analysis is never perfect.
> If you hit a false positive, a missed violation, or any unexpected behavior, please kindly
> [open a Rule Feedback issue](https://github.com/kodaskills/oplint/issues/new?template=rule_feedback.md) —
> it helps improve accuracy for everyone.

---

## ⚙️ Configuration

Generate a starter config:

```bash
oplint init            # creates .oplint.yaml
oplint init -f json    # creates .oplint.json
oplint init -f toml    # creates .oplint.toml
```

OPLint searches for `.oplint.yaml`, `.oplint.json`, or `.oplint.toml` walking up from the target directory.

### Config reference

```yaml
# File exclusions
exclude:
  use_gitignore: true         # respect .gitignore files (default: true)
  patterns: []                # gitignore-style glob patterns
  #   - node_modules          # exclude a directory by name
  #   - dist/                 # trailing slash = directory only
  #   - coverage/
  #   - "**/*.generated.ts"   # wildcard patterns

rules:
  enabled: all                # "all" or list of rule IDs: [SEC001, RES001]
  disabled: []                # rule IDs to disable
  skip_accuracy:              # skip all rules at a given accuracy level
    - approximate             # options: approximate | exact

# Override severity for a specific rule
RES001:
  severity: warning           # error | warning | info
  disabled: false

# Add custom rules (tree-sitter queries)
custom_rules:
  - id: CUSTOM001
    name: No TODO in production
    category: General
    severity: warning
    message: "TODO comment found"
    query: '(comment) @c (#match? @c "TODO")'
    expect: match
    path_filter: "**/*.ts"
    except_in:
      - "**/tests/**"
```

### Excluding files and directories

By default OPLint reads the project's `.gitignore` and skips everything it ignores. Add `patterns` for extra exclusions using the same gitignore glob syntax:

```yaml
exclude:
  use_gitignore: true   # set to false to ignore .gitignore entirely
  patterns:
    - node_modules      # directory name — excludes node_modules/ anywhere in the tree
    - dist/             # trailing slash — directory only
    - "**/*.min.js"     # wildcard — skip minified files
    - coverage/
```

| Pattern | What it excludes |
|---------|-----------------|
| `node_modules` | any directory named `node_modules` at any depth |
| `dist/` | directory named `dist` (not files) |
| `**/*.generated.ts` | all `.generated.ts` files recursively |
| `src/vendor/` | `vendor/` directory inside `src/` only |

### `skip_accuracy` examples

```yaml
# Disable all approximate rules (reduce noise, fewer false positives)
rules:
  skip_accuracy: [approximate]

# Only run exact rules (strictest, no false positives)
rules:
  skip_accuracy: [approximate]
```

---

## 🔧 Custom Rules

Rules use [tree-sitter](https://tree-sitter.github.io) queries against the TypeScript or JSON AST.

```yaml
custom_rules:
  - id: MY001
    name: No eval usage
    category: Security
    severity: error
    message: "Avoid eval() — it executes arbitrary code"
    suggestion: "Rewrite without eval"
    query: '(call_expression function: (identifier) @f (#eq? @f "eval"))'
    expect: match          # "match" = flag when found | "not-match" = flag when absent
    accuracy: exact        # approximate | exact
    accuracy_note: "Detects direct eval() calls only. Aliased eval is not detected."
```

#### `expect` semantics

| Value | Fires when |
|-------|-----------|
| `match` (default) | Query matches — pattern found in code |
| `not-match` | Query does not match — required pattern is absent |

#### `applies_to`

| Value | Target |
|-------|--------|
| _(omitted)_ | TypeScript / JavaScript files |
| `manifest` | `manifest.json` |

---

## 📊 How the Compliance Score Works

Every lint run produces a score from 0 to 100 and a grade. Here is exactly how it is computed — no magic numbers hidden.

### What we measure

The score answers: **"what fraction of the active rules is this codebase violating, and how badly?"**

It does **not** measure project size. A 1-file plugin and a 100-file plugin are judged by the same standard.

### Violation weights

Each rule has a severity. Every violation of that rule costs penalty points:

| Severity | Weight |
|----------|--------|
| `error`   | 10 |
| `warning` | 5  |
| `info`    | 1  |

### Concentration matters

A rule firing 50 times in **one file** is worse than the same rule firing once in **50 files** — the first signals a systemic problem in one place, the second is one team convention to fix everywhere.

We capture this with a logarithm. For each violated rule we look at the **maximum number of times it fires in a single file** (`max_occ`), then compute:

```
penalty(rule) = weight × (1 + log₂(max_occ))
```

Examples:

| max_occ | multiplier | error penalty | warning penalty |
|---------|-----------|---------------|-----------------|
| 1       | 1.0×      | 10            | 5               |
| 2       | 2.0×      | 20            | 10              |
| 4       | 3.0×      | 30            | 15              |
| 8       | 4.0×      | 40            | 20              |
| 50      | ≈ 6.6×    | 66            | 33              |

Rules that never fire contribute **zero** penalty regardless of how many times they _could_ fire.

### The denominator

```
total_active_weight = Σ weight(severity)  for every enabled rule
```

This is the theoretical maximum penalty if every active rule were violated at least once. It scales with your rule configuration, not your file count.

### Final formula

```
total_penalty = Σ penalty(rule)  for every rule violated ≥ once
score         = round(100 × (1 − total_penalty / total_active_weight))
score         = clamp(score, 0, 100)
```

### Grade table

| Score | Grade | Label     |
|-------|-------|-----------|
| 100   | A+    | Perfect   |
| 90–99 | A     | Excellent |
| 80–89 | B     | Good      |
| 70–79 | C     | Fair      |
| 60–69 | D     | Poor      |
| 0–59  | F     | Critical  |

**A+ means zero violations** — no rounding, no grace margin. A score of 99 is grade A.

### Why these choices

| Choice | Reason |
|--------|--------|
| Log scale for concentration | Linear accumulation would let one widespread pattern (e.g. `innerHTML` in every file) dominate the entire score and make it unreadable. Log dampens repetition while still penalising it. |
| Denominator = rule weights, not file count | File count is a project size metric, not a quality metric. Two projects with the same violation pattern should get the same score. |
| A+ only at 100 | A linter score of "A+" should mean clean — not "clean enough". Any violation, however minor, is a real finding. |
| Warnings weight 5 (not 3) | Obsidian community guidelines treat warnings seriously — most are patterns that will cause problems at scale or during review. A single unfixed warning should visibly affect the grade. |

### What the score does not capture

- **Severity of impact** beyond the three tiers — a SQL injection and a missing `onunload` both count as `error`.
- **Category weighting** — a security violation and a style violation of the same severity cost the same.
- **Accuracy** — `approximate` rules (which may have false positives) count the same as `exact` rules. Filter approximate rules in the report if you want a stricter baseline.

These are deliberate simplifications. The score is a quick signal, not a security audit.

---

## 📄 License

MIT — see [LICENSE](LICENSE) for details.

---

<div align="center">

**Maintained with ⚡ by the [Kodaskills](https://github.com/kodaskills) team**

[![Rust](https://img.shields.io/badge/Made%20with-Rust-CE422B?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)

</div>

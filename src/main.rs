use clap::{Parser, Subcommand};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use oplint::config::ConfigLoader;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "oplint")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Lint {
        path: Option<PathBuf>,
        /// Output format(s). Repeat or comma-separate: -f html -f json  or  -f html,json
        #[arg(short = 'f', long = "format", default_value = "terminal", num_args = 1.., value_delimiter = ',')]
        formats: Vec<String>,
        #[arg(short = 'c', long = "config")]
        config: Option<PathBuf>,
        #[arg(long = "no-progress", default_value_t = false)]
        no_progress: bool,
        /// Disable colors in terminal and table output
        #[arg(long = "no-color", default_value_t = false)]
        no_color: bool,
        /// Directory to write output files when multiple formats are requested
        #[arg(short = 'o', long = "output-dir")]
        output_dir: Option<PathBuf>,
    },
    Rules,
    Init {
        path: Option<PathBuf>,
        #[arg(short = 'f', long = "format", default_value = "yaml")]
        format: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Lint {
            path,
            formats,
            config,
            no_progress,
            no_color,
            output_dir,
        } => {
            let target = path.unwrap_or_else(|| PathBuf::from("."));
            lint_target(
                &target,
                &formats,
                config,
                no_progress,
                no_color,
                output_dir.as_deref(),
            );
        }
        Commands::Rules => list_rules(),
        Commands::Init { path, format } => {
            init_config(&path.unwrap_or_else(|| PathBuf::from(".")), &format)
        }
    }
}

fn lint_target(
    path: &Path,
    formats: &[String],
    config_path: Option<PathBuf>,
    no_progress: bool,
    no_color: bool,
    output_dir: Option<&Path>,
) {
    #[cfg(any(feature = "fmt-table", feature = "fmt-terminal"))]
    if no_color {
        colored::control::set_override(false);
    }

    use oplint::file_helper::{find_manifest, get_ts_files, read_file};
    use oplint::linter::Linter;

    let final_config_path: Option<PathBuf> = if let Some(cp) = config_path {
        Some(cp.to_path_buf())
    } else {
        ConfigLoader::find_config(path)
    };

    let user_config = ConfigLoader::load_or_default(final_config_path.as_deref());
    let exclude = &user_config.exclude;

    use std::time::Instant;

    let mut linter = Linter::new_with_config(final_config_path.as_deref());

    let show_progress = !no_progress;
    let pb = if show_progress {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Initializing...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    let mut violations = Vec::new();
    let mut summary = oplint::Summary::new();
    summary.plugin_name = None;
    summary.set_rule_baseline(linter.total_active_weight());
    let mut file_times: Vec<u64> = Vec::new();
    let total_start = Instant::now();

    if path.is_file() {
        if let Some(ref pb) = pb {
            pb.set_length(1);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            pb.set_message(
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "file".to_string()),
            );
        }
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let is_manifest = file_name == "manifest.json";
        let is_license = matches!(
            file_name,
            "LICENSE" | "LICENSE.md" | "LICENSE.txt" | "LICENCE" | "LICENCE.txt"
        );

        if is_manifest {
            if let Some(content) = read_file(path) {
                linter.set_is_desktop_only(Linter::detect_desktop_only(&content));
                if let Some(id) = parse_plugin_id(&content) {
                    linter.set_plugin_id(&id);
                }
                if let Some(name) = parse_plugin_name(&content) {
                    linter.set_plugin_name(&name);
                    summary.plugin_name = Some(name);
                }
                let t = Instant::now();
                let vs = linter.lint_manifest(path, &content);
                file_times.push(t.elapsed().as_millis() as u64);
                for v in &vs {
                    summary.add_violation(v);
                }
                violations.extend(vs);
            }
        } else if is_license {
            if let Some(content) = read_file(path) {
                let t = Instant::now();
                let vs = linter.lint_license(path, &content);
                file_times.push(t.elapsed().as_millis() as u64);
                for v in &vs {
                    summary.add_violation(v);
                }
                violations.extend(vs);
            }
        } else if let Some(content) = read_file(path) {
            let t = Instant::now();
            let vs = linter.lint_file(path, &content);
            file_times.push(t.elapsed().as_millis() as u64);
            for v in &vs {
                summary.add_violation(v);
            }
            violations.extend(vs);
        }
        if let Some(ref pb) = pb {
            pb.inc(1);
        }
        summary.total_files = 1;
    } else {
        // Read manifest first to detect desktop-only before linting TS files
        let manifest_path = find_manifest(path);
        let manifest_content: Option<(std::path::PathBuf, String)> = manifest_path
            .as_ref()
            .and_then(|mp| read_file(mp).map(|c| (mp.clone(), c)));

        if let Some((_, ref content)) = manifest_content {
            linter.set_is_desktop_only(Linter::detect_desktop_only(content));
            if let Some(id) = parse_plugin_id(content) {
                linter.set_plugin_id(&id);
            }
            if let Some(name) = parse_plugin_name(content) {
                linter.set_plugin_name(&name);
                summary.plugin_name = Some(name);
            }
        }

        let files = get_ts_files(path, exclude);
        let license_path = find_license(path);

        let mut total_files_count = files.len();
        if manifest_content.is_some() {
            total_files_count += 1;
        }
        if license_path.is_some() {
            total_files_count += 1;
        }

        if let Some(ref pb) = pb {
            pb.set_length(total_files_count as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            pb.set_message("Linting files...");
        }

        let file_results: Vec<_> = if let Some(ref pb) = pb {
            files
                .par_iter()
                .progress_with(pb.clone())
                .filter_map(|f| {
                    let content = read_file(f)?;
                    let t = Instant::now();
                    let vs = linter.lint_file(f, &content);
                    Some((vs, t.elapsed().as_millis() as u64))
                })
                .collect()
        } else {
            files
                .par_iter()
                .filter_map(|f| {
                    let content = read_file(f)?;
                    let t = Instant::now();
                    let vs = linter.lint_file(f, &content);
                    Some((vs, t.elapsed().as_millis() as u64))
                })
                .collect()
        };

        for (vs, elapsed) in file_results {
            file_times.push(elapsed);
            for v in &vs {
                summary.add_violation(v);
            }
            violations.extend(vs);
        }

        if let Some((mp, content)) = manifest_content {
            if let Some(ref pb) = pb {
                pb.set_message("manifest.json");
            }
            let t = Instant::now();
            let vs = linter.lint_manifest(&mp, &content);
            file_times.push(t.elapsed().as_millis() as u64);
            for v in &vs {
                summary.add_violation(v);
            }
            violations.extend(vs);
            if let Some(ref pb) = pb {
                pb.inc(1);
            }
        }

        if let Some(lp) = license_path {
            if let Some(ref pb) = pb {
                pb.set_message("LICENSE");
            }
            if let Some(content) = read_file(&lp) {
                let t = Instant::now();
                let vs = linter.lint_license(&lp, &content);
                file_times.push(t.elapsed().as_millis() as u64);
                for v in &vs {
                    summary.add_violation(v);
                }
                violations.extend(vs);
            }
            if let Some(ref pb) = pb {
                pb.inc(1);
            }
        }

        summary.total_files = total_files_count;
    }

    summary.partial_coverage = linter.partial_coverage;
    summary.all_categories = linter.all_categories();
    summary.set_timing(total_start.elapsed().as_millis() as u64, &file_times);
    summary.finalize(&violations);
    if let Some(ref pb) = pb {
        pb.finish_and_clear();
    }
    dispatch_formats(formats, output_dir, &violations, &summary);
}

fn find_license(dir: &Path) -> Option<std::path::PathBuf> {
    for name in &[
        "LICENSE",
        "LICENSE.md",
        "LICENSE.txt",
        "LICENCE",
        "LICENCE.txt",
    ] {
        let p = dir.join(name);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn parse_plugin_id(manifest_content: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(manifest_content).ok()?;
    v.get("id")?.as_str().map(|s| s.to_string())
}

fn parse_plugin_name(manifest_content: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(manifest_content).ok()?;
    v.get("name")?.as_str().map(|s| s.to_string())
}

fn list_rules() {
    use oplint::load_default_rules;
    for rule in load_default_rules() {
        println!(
            "[{}] {} - {} ({})",
            rule.id,
            rule.category,
            rule.name,
            rule.severity.as_str()
        );
        println!("    {}", rule.message);
        if let Some(suggestion) = &rule.suggestion {
            println!("    Suggestion: {}", suggestion);
        }
        println!();
    }
}

fn format_extension(fmt: &str) -> &'static str {
    match fmt {
        "json" => "json",
        "yaml" => "yaml",
        "toml" => "toml",
        "html" => "html",
        "markdown" | "md" => "md",
        "table" => "table.log",
        _ => "log",
    }
}

fn dispatch_formats(
    formats: &[String],
    output_dir: Option<&Path>,
    violations: &[oplint::Violation],
    summary: &oplint::Summary,
) {
    let to_files = formats.len() > 1 || output_dir.is_some();
    let dir = output_dir.unwrap_or(Path::new("."));

    if let Some(d) = output_dir {
        if !d.exists() {
            if let Err(e) = std::fs::create_dir_all(d) {
                log_dir_err(d, &e);
                return;
            }
        }
    }

    for fmt in formats {
        if to_files {
            let ext = format_extension(fmt);
            let filename = format!("report.{ext}");
            let path = dir.join(&filename);
            match std::fs::File::create(&path) {
                Ok(mut file) => {
                    log_write_ok(fmt, &path);
                    render_format(fmt, violations, summary, &mut file);
                }
                Err(e) => log_write_err(fmt, &path, &e),
            }
        } else {
            render_format(fmt, violations, summary, &mut std::io::stdout());
        }
    }
}

#[cfg(any(feature = "fmt-table", feature = "fmt-terminal"))]
fn log_write_ok(fmt: &str, path: &Path) {
    use colored::Colorize;
    eprintln!(
        "  {}  {:<8}  →  {}",
        "✓".green().bold(),
        fmt.bold(),
        path.display().to_string().dimmed()
    );
}

#[cfg(not(any(feature = "fmt-table", feature = "fmt-terminal")))]
fn log_write_ok(fmt: &str, path: &Path) {
    eprintln!("  ✓  {:<8}  →  {}", fmt, path.display());
}

#[cfg(any(feature = "fmt-table", feature = "fmt-terminal"))]
fn log_write_err(fmt: &str, path: &Path, e: &std::io::Error) {
    use colored::Colorize;
    eprintln!(
        "  {}  {:<8}  →  {}  {}",
        "✗".red().bold(),
        fmt.bold(),
        path.display(),
        format!("({e})").red()
    );
}

#[cfg(not(any(feature = "fmt-table", feature = "fmt-terminal")))]
fn log_write_err(fmt: &str, path: &Path, e: &std::io::Error) {
    eprintln!("  ✗  {:<8}  →  {}  ({e})", fmt, path.display());
}

#[cfg(any(feature = "fmt-table", feature = "fmt-terminal"))]
fn log_dir_err(dir: &Path, e: &std::io::Error) {
    use colored::Colorize;
    eprintln!(
        "  {}  Cannot create output dir: {}  {}",
        "✗".red().bold(),
        dir.display(),
        format!("({e})").red()
    );
}

#[cfg(not(any(feature = "fmt-table", feature = "fmt-terminal")))]
fn log_dir_err(dir: &Path, e: &std::io::Error) {
    eprintln!("  ✗  Cannot create output dir: {}  ({e})", dir.display());
}

fn render_format(
    format: &str,
    violations: &[oplint::Violation],
    summary: &oplint::Summary,
    out: &mut dyn std::io::Write,
) {
    #[allow(unused_variables)]
    let _ = (format, violations, summary);
    match format {
        #[cfg(feature = "fmt-json")]
        "json" => oplint::format_json(violations, summary, out),
        #[cfg(feature = "fmt-yaml")]
        "yaml" => oplint::format_yaml(violations, summary, out),
        #[cfg(feature = "fmt-toml")]
        "toml" => oplint::format_toml(violations, summary, out),
        #[cfg(feature = "fmt-html")]
        "html" => oplint::format_html(violations, summary, out),
        #[cfg(feature = "fmt-markdown")]
        "markdown" | "md" => oplint::format_markdown(violations, summary, out),
        #[cfg(feature = "fmt-table")]
        "table" => oplint::format_table(violations, summary, out),
        #[cfg(feature = "fmt-terminal")]
        _ => oplint::format_terminal(violations, summary, out),
        #[cfg(not(feature = "fmt-terminal"))]
        _ => eprintln!(
            "Format '{}' not available. Enable the corresponding fmt-* feature.",
            format
        ),
    }
}

fn init_config(dir: &Path, format: &str) {
    match format {
        "yaml" | "yml" => {
            let config = include_str!("../templates/config.yaml");
            std::fs::write(dir.join(".oplint.yaml"), config).expect("Failed to write config");
            println!("Created: .oplint.yaml");
        }
        "json" => {
            let config = include_str!("../templates/config.json");
            std::fs::write(dir.join(".oplint.json"), config).expect("Failed to write config");
            println!("Created: .oplint.json");
        }
        "toml" => {
            let config = include_str!("../templates/config.toml");
            std::fs::write(dir.join(".oplint.toml"), config).expect("Failed to write config");
            println!("Created: .oplint.toml");
        }
        _ => {
            eprintln!("Unsupported format: {}. Use yaml, json, or toml.", format);
        }
    }
}

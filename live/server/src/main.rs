use askama::Template;
use axum::{
    extract::Json,
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Deserialize)]
struct LintRequest {
    provider: String,
    owner: String,
    repo: String,
    config: Option<String>,
    formats: Option<Vec<String>>,
}

#[derive(Serialize)]
struct LintResponse {
    output: String,
    html: Option<String>,
    score: Option<f64>,
    grade: Option<String>,
    error: Option<String>,
    reports: HashMap<String, String>,
    issues_count: Option<usize>,
    lint_duration_ms: Option<u64>,
    files_analyzed: Option<usize>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

async fn index() -> Html<String> {
    let tmpl = IndexTemplate {};
    Html(tmpl.render().unwrap_or_else(|e| e.to_string()))
}

fn validate_formats(formats: &[String]) -> Vec<String> {
    let valid_formats = [
        "json", "toml", "yaml", "markdown", "terminal", "table", "html",
    ];
    let mut result: Vec<String> = formats
        .iter()
        .filter(|f| valid_formats.contains(&f.as_str()))
        .cloned()
        .collect();

    if result.is_empty() {
        result.push("json".to_string());
    }

    result.sort();
    result.dedup();
    result
}

fn get_report_filename(format: &str) -> String {
    match format {
        "json" => "report.json".to_string(),
        "toml" => "report.toml".to_string(),
        "yaml" => "report.yaml".to_string(),
        "markdown" => "report.md".to_string(),
        "terminal" => "report.log".to_string(),
        "table" => "report.table.log".to_string(),
        "html" => "report.html".to_string(),
        _ => format!("report.{}", format),
    }
}

async fn lint_handler(Json(mut req): Json<LintRequest>) -> (StatusCode, Json<LintResponse>) {
    let provider = req.provider.to_lowercase();
    let owner = &req.owner;
    let repo = &req.repo;

    let formats = req
        .formats
        .take()
        .map(|f| validate_formats(&f))
        .unwrap_or_else(|| vec!["json".to_string()]);

    let clone_url = match provider.as_str() {
        "github" => format!("https://github.com/{}/{}.git", owner, repo),
        "gitlab" => format!("https://gitlab.com/{}/{}.git", owner, repo),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(LintResponse {
                    output: "".to_string(),
                    html: None,
                    score: None,
                    grade: None,
                    error: Some(format!("Unsupported provider: {}", provider)),
                    reports: HashMap::new(),
                    issues_count: None,
                    lint_duration_ms: None,
                    files_analyzed: None,
                }),
            );
        }
    };

    let pid = std::process::id();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let repo_dir = std::env::temp_dir().join(format!("oplint-{}-{}", pid, timestamp));
    let repo_dir_str = repo_dir.to_string_lossy().to_string();

    let clone_result = Command::new("git")
        .args(["clone", "--depth", "1", &clone_url, &repo_dir_str])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match clone_result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let _ = std::fs::remove_dir_all(&repo_dir);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(LintResponse {
                        output: "".to_string(),
                        html: None,
                        score: None,
                        grade: None,
                        error: Some(format!("Failed to clone repo: {}", stderr)),
                        reports: HashMap::new(),
                        issues_count: None,
                        lint_duration_ms: None,
                        files_analyzed: None,
                    }),
                );
            }
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&repo_dir);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LintResponse {
                    output: "".to_string(),
                    html: None,
                    score: None,
                    grade: None,
                    error: Some(format!("Failed to run git clone: {}", e)),
                    reports: HashMap::new(),
                    issues_count: None,
                    lint_duration_ms: None,
                    files_analyzed: None,
                }),
            );
        }
    }

    if let Some(ref config) = req.config {
        let config_path = Path::new(&repo_dir_str).join(".oplint.yaml");
        if let Err(e) = std::fs::write(&config_path, config) {
            let _ = std::fs::remove_dir_all(&repo_dir);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LintResponse {
                    output: "".to_string(),
                    html: None,
                    score: None,
                    grade: None,
                    error: Some(format!("Failed to write config: {}", e)),
                    reports: HashMap::new(),
                    issues_count: None,
                    lint_duration_ms: None,
                    files_analyzed: None,
                }),
            );
        }
    }

    let oplint_paths = vec![
        "oplint".to_string(),
        "/usr/local/bin/oplint".to_string(),
        format!(
            "{}/.local/bin/oplint",
            std::env::var("HOME").unwrap_or_default()
        ),
    ];

    let oplint_path = {
        let mut found = None;
        for path in &oplint_paths {
            if std::path::Path::new(path).exists() {
                found = Some(path.clone());
                break;
            }
            if let Ok(output) = Command::new("which").arg(path).output().await {
                if output.status.success() {
                    let found_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !found_path.is_empty() {
                        found = Some(found_path);
                        break;
                    }
                }
            }
        }
        found
    };

    let oplint_path = match oplint_path {
        Some(p) => p,
        None => {
            let _ = std::fs::remove_dir_all(&repo_dir);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LintResponse {
                    output: "".to_string(),
                    html: None,
                    score: None,
                    grade: None,
                    error: Some(
                        "oplint binary not found. Install it: https://github.com/Kodaskills/oplint/releases".to_string(),
                    ),
                    reports: HashMap::new(),
                    issues_count: None,
                    lint_duration_ms: None,
                    files_analyzed: None,
                }),
            );
        }
    };

    let report_dir = std::env::temp_dir().join(format!("oplint-reports-{}-{}", pid, timestamp));
    let _ = std::fs::create_dir_all(&report_dir);

    let mut all_formats = formats.clone();
    if !all_formats.contains(&"json".to_string()) {
        all_formats.push("json".to_string());
    }
    let formats_str = all_formats.join(",");
    tracing::debug!(
        "Running oplint: {} lint {} -f {} -o {}",
        oplint_path,
        repo_dir_str,
        formats_str,
        report_dir.display()
    );

    let lint_start = std::time::Instant::now();

    let oplint_run = Command::new(&oplint_path)
        .args([
            "lint",
            &repo_dir_str,
            "-f",
            &formats_str,
            "-o",
            report_dir.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    let lint_duration_ms = lint_start.elapsed().as_millis() as u64;

    let oplint_run = match oplint_run {
        Ok(o) => o,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&repo_dir);
            let _ = std::fs::remove_dir_all(&report_dir);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LintResponse {
                    output: "".to_string(),
                    html: None,
                    score: None,
                    grade: None,
                    error: Some(format!("Failed to run oplint: {}", e)),
                    reports: HashMap::new(),
                    issues_count: None,
                    lint_duration_ms: Some(lint_duration_ms),
                    files_analyzed: None,
                }),
            );
        }
    };

    tracing::debug!("oplint exit status: {}", oplint_run.status);
    tracing::debug!(
        "oplint stderr: {}",
        String::from_utf8_lossy(&oplint_run.stderr)
    );

    let mut reports = HashMap::new();
    let mut json_content = None;
    let mut html_content = None;

    for format in &all_formats {
        let filename = get_report_filename(format);
        let report_path = report_dir.join(&filename);

        tracing::debug!("Reading {} from: {}", format, report_path.display());
        if let Ok(content) = std::fs::read_to_string(&report_path) {
            tracing::debug!("{} content length: {}", format, content.len());

            if format == "json" {
                json_content = Some(content.clone());
            } else if format == "html" {
                html_content = Some(content.clone());
            }

            if formats.contains(format) {
                reports.insert(format.clone(), content);
            }
        }
    }

    let _ = std::fs::remove_dir_all(&report_dir);
    let _ = std::fs::remove_dir_all(&repo_dir);

    let stderr = String::from_utf8_lossy(&oplint_run.stderr).to_string();

    let (score, grade, issues_count, files_analyzed, json_duration_ms) = json_content
        .as_deref()
        .map(|json_str| parse_report_stats(json_str))
        .unwrap_or((None, None, None, None, None));

    let error = if oplint_run.status.success() {
        None
    } else if !stderr.is_empty() {
        Some(stderr)
    } else {
        None
    };

    let output = json_content.unwrap_or_default();
    let html = html_content.filter(|s| !s.is_empty());

    (
        StatusCode::OK,
        Json(LintResponse {
            output,
            html,
            score,
            grade,
            error,
            reports,
            issues_count,
            lint_duration_ms: json_duration_ms.or(Some(lint_duration_ms)),
            files_analyzed,
        }),
    )
}

fn parse_report_stats(
    json_str: &str,
) -> (
    Option<f64>,
    Option<String>,
    Option<usize>,
    Option<usize>,
    Option<u64>,
) {
    #[derive(serde::Deserialize)]
    struct JsonOutput {
        summary: Option<JsonSummary>,
    }
    
    #[derive(serde::Deserialize)]
    struct JsonSummary {
        score: Option<f64>,
        grade: Option<String>,
        total_violations: Option<usize>,
        total_files: Option<usize>,
        duration_ms: Option<u64>,
    }
    
    match serde_json::from_str::<JsonOutput>(json_str) {
        Ok(output) => {
            if let Some(summary) = output.summary {
                tracing::debug!("Parsed stats: score={:?}, grade={:?}", summary.score, summary.grade);
                (
                    summary.score,
                    summary.grade,
                    summary.total_violations,
                    summary.total_files,
                    summary.duration_ms,
                )
            } else {
                tracing::warn!("No summary in JSON output");
                (None, None, None, None, None)
            }
        }
        Err(e) => {
            tracing::warn!("Failed to parse JSON report for stats: {}", e);
            (None, None, None, None, None)
        }
    }
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "oplint_live=debug,axum=debug".to_string()),
        )
        .init();

    let cors = tower_http::cors::CorsLayer::permissive();

    let app = Router::new()
        .route("/", get(index))
        .route("/lint", post(lint_handler))
        .layer(cors);

    let addr = std::env::var("OPLINT_LIVE_PORT").unwrap_or_else(|_| "8080".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", addr))
        .await
        .unwrap();
    tracing::info!("Listening on http://0.0.0.0:{}", addr);
    axum::serve(listener, app).await.unwrap();
}

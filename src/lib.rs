pub mod checks;
pub mod config;
pub mod file_helper;
pub mod formatters;
pub mod linter;
pub mod report;
pub mod types;

pub use config::{
    load_default_rules, ConfigLoader, CustomRule, ExcludeConfig, RuleMatcher, UserConfig,
};
pub use linter::Linter;
#[cfg(any(feature = "fmt-html", feature = "fmt-markdown"))]
pub use report::{SummaryView, ViolationGroupView};
pub use types::{OutputFormat, Rule, Severity, Summary, Violation};

#[cfg(feature = "fmt-html")]
pub use formatters::format_html;
#[cfg(feature = "fmt-json")]
pub use formatters::format_json;
#[cfg(feature = "fmt-markdown")]
pub use formatters::format_markdown;
#[cfg(feature = "fmt-table")]
pub use formatters::format_table;
#[cfg(feature = "fmt-terminal")]
pub use formatters::format_terminal;
#[cfg(feature = "fmt-toml")]
pub use formatters::format_toml;
#[cfg(feature = "fmt-yaml")]
pub use formatters::format_yaml;

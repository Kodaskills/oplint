use super::CheckContext;
use crate::types::Violation;
use std::path::Path;

/// MAN012: Validate that the LICENSE file contains a copyright notice.
/// Looks for a line matching: Copyright (c) YYYY[-YYYY] Name...
pub fn check(path: &Path, content: &str, ctx: &CheckContext<'_>) -> Vec<Violation> {
    let has_copyright = content.lines().any(|line| {
        let trimmed = line.trim();
        // Case‑insensitive check for "Copyright (c)" followed by a year and a name
        trimmed.to_lowercase().starts_with("copyright")
            && trimmed.contains("(c)")
            && trimmed.chars().any(|c| c.is_ascii_digit())
            && trimmed.len() > 20 // avoid trivial lines
    });

    if has_copyright {
        return vec![];
    }

    let mut v = Violation::new(
        ctx.rule_id,
        ctx.rule_category,
        ctx.rule_message,
        ctx.rule_severity.clone(),
        path.to_path_buf(),
        1, // we can't pinpoint a line, so use the first line as fallback
    );
    if let Some(s) = ctx.rule_suggestion {
        v = v.with_suggestion(s);
    }
    v = v.with_accuracy(ctx.accuracy, ctx.accuracy_note);
    vec![v]
}

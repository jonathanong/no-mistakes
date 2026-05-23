/// Classification of a file's content.
#[derive(Debug, PartialEq, Eq)]
pub enum ContentKind {
    /// The file is blank or contains only whitespace.
    Empty,
    /// The file contains only comments and whitespace (no real code).
    CommentsOnly,
    /// The file contains non-comment, non-whitespace content.
    HasContent,
}

/// Classify the content of a file based on its extension.
///
/// - Blank/whitespace → `Empty`
/// - Known extension where all non-blank lines are comments → `CommentsOnly`
/// - Known extension with at least one non-comment line → `HasContent`
/// - Unknown extension → always `HasContent`
pub fn classify_content(content: &str, ext: &str) -> ContentKind {
    if content.trim().is_empty() {
        return ContentKind::Empty;
    }

    match ext {
        "ts" | "mts" | "cts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" => classify_c_style(content),
        "sql" => classify_sql(content),
        "rs" | "css" => classify_c_style(content),
        "md" => classify_markdown(content),
        _ => ContentKind::HasContent,
    }
}

/// Classify content using C-style comment stripping (`//` line and `/* */` blocks).
fn classify_c_style(content: &str) -> ContentKind {
    if has_real_content_c_style(content) {
        ContentKind::HasContent
    } else {
        ContentKind::CommentsOnly
    }
}

/// Returns `true` if there is at least one non-comment, non-whitespace line.
fn has_real_content_c_style(content: &str) -> bool {
    let mut in_block = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if in_block {
            if let Some(end) = trimmed.find("*/") {
                in_block = false;
                // Check if anything follows the closing */
                let after = trimmed[end + 2..].trim();
                if !after.is_empty() && !after.starts_with("//") {
                    return true;
                }
            }
            // Still inside block comment — skip line
            continue;
        }
        if trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with("/*") {
            if trimmed.contains("*/") {
                // Block comment opens and closes on the same line
                let after_close = &trimmed[trimmed.find("*/").unwrap() + 2..].trim();
                if !after_close.is_empty() && !after_close.starts_with("//") {
                    return true;
                }
            } else {
                in_block = true;
            }
            continue;
        }
        return true;
    }
    false
}

/// Classify SQL content (line comments with `--`, block comments with `/* */`).
fn classify_sql(content: &str) -> ContentKind {
    if has_real_content_sql(content) {
        ContentKind::HasContent
    } else {
        ContentKind::CommentsOnly
    }
}

fn has_real_content_sql(content: &str) -> bool {
    let mut in_block = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if in_block {
            if let Some(end) = trimmed.find("*/") {
                in_block = false;
                let after = trimmed[end + 2..].trim();
                if !after.is_empty() && !after.starts_with("--") {
                    return true;
                }
            }
            continue;
        }
        if trimmed.starts_with("--") {
            continue;
        }
        if trimmed.starts_with("/*") {
            if trimmed.contains("*/") {
                let after_close = trimmed[trimmed.find("*/").unwrap() + 2..].trim();
                if !after_close.is_empty() && !after_close.starts_with("--") {
                    return true;
                }
            } else {
                in_block = true;
            }
            continue;
        }
        return true;
    }
    false
}

/// Classify markdown content (HTML comments `<!-- ... -->`).
fn classify_markdown(content: &str) -> ContentKind {
    if has_real_content_markdown(content) {
        ContentKind::HasContent
    } else {
        ContentKind::CommentsOnly
    }
}

fn has_real_content_markdown(content: &str) -> bool {
    let mut in_comment = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if in_comment {
            if let Some(end) = trimmed.find("-->") {
                in_comment = false;
                let after = trimmed[end + 3..].trim();
                if !after.is_empty() {
                    return true;
                }
            }
            continue;
        }
        if trimmed.starts_with("<!--") {
            if trimmed.contains("-->") {
                let after_close = trimmed[trimmed.find("-->").unwrap() + 3..].trim();
                if !after_close.is_empty() {
                    return true;
                }
            } else {
                in_comment = true;
            }
            continue;
        }
        return true;
    }
    false
}

#[cfg(test)]
mod tests;

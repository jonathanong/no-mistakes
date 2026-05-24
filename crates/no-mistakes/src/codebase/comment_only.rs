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
        "ts" | "mts" | "cts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" => {
            kind(has_real_content_block(content, "//"))
        }
        "sql" => kind(has_real_content_block(content, "--")),
        "rs" | "css" => kind(has_real_content_block(content, "//")),
        "md" => kind(has_real_content_markdown(content)),
        _ => ContentKind::HasContent,
    }
}

fn kind(has_content: bool) -> ContentKind {
    if has_content {
        ContentKind::HasContent
    } else {
        ContentKind::CommentsOnly
    }
}

/// Returns `true` if `tail` (text after a `*/`) contains real content.
/// `line_cmt` is the line-comment prefix (`"//"` or `"--"`).
/// Handles chained `/* … */` blocks correctly.
pub(crate) fn block_tail_has_content<'a>(mut tail: &'a str, line_cmt: &str) -> bool {
    loop {
        tail = tail.trim_start();
        if tail.is_empty() || tail.starts_with(line_cmt) {
            return false;
        }
        if tail.starts_with("/*") {
            if let Some(end) = tail.find("*/") {
                tail = &tail[end + 2..];
                continue;
            }
            return false;
        }
        return true;
    }
}

/// Returns `true` if `content` has at least one non-comment, non-whitespace line.
/// Supports C-style (`//` / `/* */`) and SQL-style (`--` / `/* */`) via `line_cmt`.
fn has_real_content_block(content: &str, line_cmt: &str) -> bool {
    let mut in_block = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if in_block {
            if let Some(end) = trimmed.find("*/") {
                in_block = false;
                if block_tail_has_content(trimmed[end + 2..].trim(), line_cmt) {
                    return true;
                }
            }
            continue;
        }
        if trimmed.starts_with(line_cmt) {
            continue;
        }
        if trimmed.starts_with("/*") {
            if trimmed.contains("*/") {
                let after = trimmed[trimmed.find("*/").unwrap() + 2..].trim();
                if block_tail_has_content(after, line_cmt) {
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

/// Returns `true` if `tail` (text after a `-->`) contains real markdown content.
/// Handles chained `<!-- … -->` comments correctly.
pub(crate) fn md_tail_has_content(mut tail: &str) -> bool {
    loop {
        tail = tail.trim_start();
        if tail.is_empty() {
            return false;
        }
        if tail.starts_with("<!--") {
            if let Some(end) = tail.find("-->") {
                tail = &tail[end + 3..];
                continue;
            }
            return false;
        }
        return true;
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
                if md_tail_has_content(trimmed[end + 3..].trim()) {
                    return true;
                }
            }
            continue;
        }
        if trimmed.starts_with("<!--") {
            if trimmed.contains("-->") {
                let after = trimmed[trimmed.find("-->").unwrap() + 3..].trim();
                if md_tail_has_content(after) {
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

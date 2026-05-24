fn normalize_discovery_path(path: &Path) -> PathBuf {
    let normalized = crate::codebase::ts_resolver::normalize_path(path);
    if normalized.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        normalized
    }
}

fn is_under_skipped_dir(root: &Path, path: &Path, extra_skip: &HashSet<&str>) -> bool {
    path.strip_prefix(root).ok().is_some_and(|rel| {
        rel.components().any(|component| {
            component
                .as_os_str()
                .to_str()
                .is_some_and(|name| SKIP_DIRS.contains(&name) || extra_skip.contains(name))
        })
    })
}

fn git_ls_files(root: &Path) -> Option<Vec<String>> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(root).arg("ls-files");
    cmd.env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE");
    cmd.arg("--cached").arg("--others").arg("--exclude-standard");
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8(out.stdout).ok()?;
    let mut files: Vec<String> = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    files.sort();
    files.dedup();
    Some(files)
}

pub fn byte_offset_to_line(source: &str, byte_offset: usize) -> u32 {
    let end = byte_offset.min(source.len());
    let line = source[..end].bytes().filter(|&b| b == b'\n').count();
    (line + 1) as u32
}

/// Returns `true` if the line immediately before `stmt_line` (1-based) contains
/// a `no-mistakes-disable-next-line <rule_id>` directive comment.
///
/// Matches:
/// - `// no-mistakes-disable-next-line <rule_id>`
/// - `// no-mistakes-disable-next-line <rule_id>: <reason>`
/// - `// no-mistakes-disable-next-line <rule_id> <reason>`
pub fn has_disable_comment(source: &str, stmt_line: u32, rule_id: &str) -> bool {
    if stmt_line < 2 {
        return false;
    }
    source
        .lines()
        .nth((stmt_line - 2) as usize)
        .map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("//") {
                return false;
            }
            let rest = trimmed
                .strip_prefix("//")
                .expect("line starts with //")
                .trim();
            let Some(after_directive) = rest.strip_prefix("no-mistakes-disable-next-line ") else {
                return false;
            };
            let rule_part = after_directive.trim();
            rule_part.strip_prefix(rule_id).is_some_and(|suffix| {
                suffix.is_empty()
                    || suffix.starts_with(':')
                    || suffix.starts_with(char::is_whitespace)
            })
        })
        .unwrap_or(false)
}

/// Returns `true` if `stmt_line` (1-based) contains a
/// `no-mistakes-disable-line <rule_id>` directive comment.
///
/// Matches:
/// - `// no-mistakes-disable-line <rule_id>`
/// - `// no-mistakes-disable-line <rule_id>: <reason>`
/// - `// no-mistakes-disable-line <rule_id> <reason>`
pub fn has_disable_line_comment(source: &str, stmt_line: u32, rule_id: &str) -> bool {
    if stmt_line == 0 {
        return false;
    }
    source
        .lines()
        .nth((stmt_line - 1) as usize)
        .is_some_and(|line| line_comment_directive_matches(line, "no-mistakes-disable-line ", rule_id))
}

/// Returns `true` if a leading comment disables `rule_id` for the whole file.
///
/// Matches:
/// - `// no-mistakes-disable-file <rule_id>`
/// - `// no-mistakes-disable-file <rule_id>: <reason>`
/// - `// no-mistakes-disable-file <rule_id> <reason>`
pub fn has_disable_file_comment(source: &str, rule_id: &str) -> bool {
    let mut in_block_comment = false;

    for line in source.trim_start_matches('\u{FEFF}').lines() {
        let mut rest = line.trim();

        loop {
            if rest.is_empty() {
                break;
            }

            if in_block_comment {
                let Some(end) = rest.find("*/") else {
                    break;
                };
                in_block_comment = false;
                rest = rest[end + 2..].trim();
                continue;
            }

            if rest.starts_with("/*") {
                let Some(end) = rest.find("*/") else {
                    in_block_comment = true;
                    break;
                };
                rest = rest[end + 2..].trim();
                continue;
            }

            let Some(rest) = rest.strip_prefix("//").map(|s| s.trim()) else {
                return false;
            };
            let Some(after_directive) = rest.strip_prefix("no-mistakes-disable-file ") else {
                break;
            };
            let rule_part = after_directive.trim();
            if rule_part.strip_prefix(rule_id).is_some_and(|suffix| {
                suffix.is_empty()
                    || suffix.starts_with(':')
                    || suffix.starts_with(char::is_whitespace)
            }) {
                return true;
            }
            break;
        }
    }

    false
}

fn line_comment_directive_matches(line: &str, directive: &str, rule_id: &str) -> bool {
    let Some(comment_start) = line_comment_start(line) else {
        return false;
    };
    let rest = line[comment_start + 2..].trim();
    let Some(after_directive) = rest.strip_prefix(directive) else {
        return false;
    };
    rule_part_matches(after_directive.trim(), rule_id)
}

fn line_comment_start(line: &str) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    let mut in_block_comment = false;
    let mut chars = line.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        if in_block_comment {
            if ch == '*' && chars.peek().is_some_and(|(_, next)| *next == '/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }
        if let Some(current) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == current {
                quote = None;
            }
            continue;
        }
        if matches!(ch, '\'' | '"' | '`') {
            quote = Some(ch);
            continue;
        }
        if ch == '/' && chars.peek().is_some_and(|(_, next)| *next == '*') {
            chars.next();
            in_block_comment = true;
            continue;
        }
        if ch == '/' && chars.peek().is_some_and(|(_, next)| *next == '/') {
            return Some(idx);
        }
    }
    None
}

fn rule_part_matches(rule_part: &str, rule_id: &str) -> bool {
    rule_part.strip_prefix(rule_id).is_some_and(|suffix| {
        suffix.is_empty() || suffix.starts_with(':') || suffix.starts_with(char::is_whitespace)
    })
}

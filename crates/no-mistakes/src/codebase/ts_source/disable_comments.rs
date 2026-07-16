pub(crate) fn normalize_discovery_path(path: &Path) -> PathBuf {
    let normalized = crate::codebase::ts_resolver::normalize_path(path);
    if normalized.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        normalized
    }
}

pub(crate) fn is_under_skipped_dir(root: &Path, path: &Path, extra_skip: &HashSet<&str>) -> bool {
    path.strip_prefix(root).ok().is_some_and(|rel| {
        rel.components().any(|component| {
            let name = component.as_os_str().to_str();
            name.is_some_and(|name| SKIP_DIRS.contains(&name) || extra_skip.contains(name))
        })
    })
}

struct DiscoveredPathViews {
    visible: Vec<PathBuf>,
    tracked: Vec<PathBuf>,
}

fn git_ls_path_views(root: &Path) -> Option<DiscoveredPathViews> {
    let mut cmd = Command::new("git");
    cmd.current_dir(root);
    cmd.arg("ls-files").arg("-z").arg("-t");
    cmd.env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE");
    cmd.arg("--cached")
        .arg("--others")
        .arg("--exclude-standard");
    let out = crate::invocation::command_output(&mut cmd).ok()?;
    if !out.status.success() {
        return None;
    }
    Some(parse_git_tagged_paths(&out.stdout))
}

fn parse_git_tagged_paths(output: &[u8]) -> DiscoveredPathViews {
    let mut visible = Vec::new();
    let mut tracked = Vec::new();
    for record in output
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let [tag, b' ', path @ ..] = record else {
            continue;
        };
        if path.is_empty() {
            continue;
        }
        let path = git_output_path(path);
        visible.push(path.clone());
        if !matches!(*tag, b'?' | b'K') {
            tracked.push(path);
        }
    }
    visible.sort();
    visible.dedup();
    tracked.sort();
    tracked.dedup();
    DiscoveredPathViews { visible, tracked }
}

#[cfg(unix)]
fn git_output_path(bytes: &[u8]) -> PathBuf {
    use std::os::unix::ffi::OsStringExt;
    std::ffi::OsString::from_vec(bytes.to_vec()).into()
}

#[cfg(not(unix))]
fn git_output_path(bytes: &[u8]) -> PathBuf {
    String::from_utf8_lossy(bytes).into_owned().into()
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
        .trim_start_matches('\u{FEFF}')
        .lines()
        .nth((stmt_line - 2) as usize)
        .map(|line| {
            let trimmed = line.trim();
            let Some(rest) = leading_comment_text(trimmed) else {
                return false;
            };
            let Some(after_directive) = rest.strip_prefix("no-mistakes-disable-next-line ") else {
                return false;
            };
            rule_part_matches(after_directive.trim(), rule_id)
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
    line_comment_directive_matches(source, stmt_line, "no-mistakes-disable-line ", rule_id)
}

/// Returns `true` if a leading comment disables `rule_id` for the whole file.
///
/// Matches:
/// - `// no-mistakes-disable-file <rule_id>`
/// - `// no-mistakes-disable-file <rule_id>: <reason>`
/// - `// no-mistakes-disable-file <rule_id> <reason>`
pub fn has_disable_file_comment(source: &str, rule_id: &str) -> bool {
    let mut in_block_comment = false;
    let mut saw_hash_attribute = false;

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

            let comment_prefix_is_slash = rest.starts_with("//");
            saw_hash_attribute |= hash_attribute_comment_line(rest);
            let Some(rest) = leading_comment_text(rest) else {
                return false;
            };
            let Some(after_directive) = rest.strip_prefix("no-mistakes-disable-file ") else {
                break;
            };
            if saw_hash_attribute && comment_prefix_is_slash {
                return false;
            }
            let rule_part = after_directive.trim();
            if rule_part_matches(rule_part, rule_id) {
                return true;
            }
            break;
        }
    }

    false
}

fn hash_attribute_comment_line(line: &str) -> bool {
    line.starts_with("#![")
        || line
            .strip_prefix("#[")
            .and_then(|rest| rest.chars().next())
            .is_some_and(is_word_char)
}

fn line_comment_directive_matches(
    source: &str,
    stmt_line: u32,
    directive: &str,
    rule_id: &str,
) -> bool {
    let mut state = LineCommentScanState::default();
    for (index, line) in source.trim_start_matches('\u{FEFF}').lines().enumerate() {
        let line_number = (index + 1) as u32;
        let comment = line_comment_start(line, &mut state);

        if line_number != stmt_line {
            continue;
        }

        let Some((comment_start, prefix_len)) = comment else {
            return false;
        };
        let rest = line[comment_start + prefix_len..].trim();
        let Some(after_directive) = rest.strip_prefix(directive) else {
            return false;
        };
        return rule_part_matches(after_directive.trim(), rule_id);
    }

    false
}

fn rule_part_matches(rule_part: &str, rule_id: &str) -> bool {
    rule_part.strip_prefix(rule_id).is_some_and(|suffix| {
        suffix.is_empty() || suffix.starts_with(':') || suffix.starts_with(char::is_whitespace)
    })
}

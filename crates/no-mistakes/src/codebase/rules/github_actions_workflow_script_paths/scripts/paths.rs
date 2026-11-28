use super::super::options::{matches_script_dir, prefixes_script_dir};
use super::{ScriptHit, ScriptHitKind, WorkingDir};
use crate::codebase::rules::github_actions_download_integrity::words::Word;

pub(super) fn check_candidate(
    word: &Word,
    command_dynamic: bool,
    dir: &WorkingDir,
    script_dirs: &[String],
    line_offset: usize,
    hits: &mut Vec<ScriptHit>,
) {
    let line = word.line + line_offset;
    let raw = word.text.trim();
    if raw.is_empty() {
        return;
    }
    if filename_glob_targets(raw, dir, script_dirs) {
        hits.push(unanalyzable(
            line,
            raw.to_string(),
            format!(
                "invokes shell filename pattern `{raw}`, which cannot be proven against workflow path filters. Pass a literal repository script path."
            ),
        ));
        return;
    }
    if is_dynamic_word(word, command_dynamic) {
        if dynamic_may_target_script_dir(raw, dir, script_dirs) {
            hits.push(unanalyzable(
                line,
                raw.to_string(),
                format!(
                    "invokes dynamic script path `{raw}`, which cannot be proven against workflow path filters. Pass a literal repository script path."
                ),
            ));
        }
        return;
    }
    match dir {
        WorkingDir::Known(base) => {
            if let Some(resolved) = resolve_rel_path(base, raw) {
                if matches_script_dir(&resolved, script_dirs) {
                    hits.push(ScriptHit {
                        line,
                        kind: ScriptHitKind::Invocation {
                            path: resolved,
                            token: raw.to_string(),
                        },
                    });
                }
            }
        }
        WorkingDir::Dynamic(raw_dir) => {
            if !raw.starts_with('/') && !raw.starts_with('~') {
                hits.push(unanalyzable(
                    line,
                    raw.to_string(),
                    format!(
                        "runs `{raw}` under dynamic working directory `{raw_dir}`, which cannot be proven against workflow path filters. Use a literal working-directory or repository-root script path."
                    ),
                ));
            }
        }
        WorkingDir::OutsideRepo => {}
    }
}

pub(in super::super) fn resolve_rel_path(base: &str, raw: &str) -> Option<String> {
    if raw.starts_with('/') || raw.starts_with('~') || raw.contains("://") {
        return None;
    }
    let mut parts: Vec<&str> = Vec::new();
    for source in [base, raw] {
        for segment in source.split('/') {
            if segment.is_empty() || segment == "." {
                continue;
            }
            if segment == ".." {
                parts.pop()?;
            } else {
                parts.push(segment);
            }
        }
    }
    Some(parts.join("/"))
}

pub(super) fn is_dynamic_word(word: &Word, command_dynamic: bool) -> bool {
    word.bare_expansion.is_some()
        || word.quoted_expansion.is_some()
        || word.expansion_may_split
        || word.text.contains("${{")
        || (command_dynamic && (word.text.contains('$') || word.text.contains('`')))
}

pub(super) fn unanalyzable(line: usize, token: String, detail: String) -> ScriptHit {
    ScriptHit {
        line,
        kind: ScriptHitKind::Unanalyzable { token, detail },
    }
}

fn filename_glob_targets(raw: &str, dir: &WorkingDir, script_dirs: &[String]) -> bool {
    let Some(index) = raw.find(['*', '?', '[']) else {
        return false;
    };
    let head = raw[..index].trim_end_matches('/');
    if head.is_empty() {
        return false;
    }
    if raw.starts_with('/') || raw.starts_with('~') {
        return false;
    }
    match dir {
        WorkingDir::OutsideRepo => false,
        WorkingDir::Dynamic(_) => true,
        WorkingDir::Known(base) => resolve_rel_path(base, head).is_some_and(|resolved| {
            script_dirs.iter().any(|dir| {
                dir == &resolved
                    || dir.starts_with(&format!("{resolved}/"))
                    || resolved.starts_with(&format!("{dir}/"))
            })
        }),
    }
}

fn dynamic_may_target_script_dir(raw: &str, dir: &WorkingDir, script_dirs: &[String]) -> bool {
    if raw.starts_with('/') || raw.starts_with('~') {
        return false;
    }
    match dir {
        WorkingDir::OutsideRepo => false,
        WorkingDir::Dynamic(_) => true,
        WorkingDir::Known(base) => dynamic_known_may_target(raw, base, script_dirs),
    }
}

fn dynamic_known_may_target(raw: &str, base: &str, script_dirs: &[String]) -> bool {
    if !base.is_empty() && prefixes_script_dir(base, script_dirs) {
        return true;
    }
    let head = raw.split(['$', '{']).next().unwrap_or("");
    let prefix = head.trim_end_matches('/');
    if prefix.is_empty() || prefix == "." {
        return true;
    }
    let Some(resolved) = resolve_rel_path(base, prefix) else {
        return false;
    };
    if prefixes_script_dir(&resolved, script_dirs)
        || script_dirs
            .iter()
            .any(|dir| dir.starts_with(&format!("{resolved}/")))
    {
        return true;
    }
    // A partial segment such as `scr${part}ipts` can still name `scripts`.
    !head.ends_with('/') && script_dirs.iter().any(|dir| dir.starts_with(&resolved))
}

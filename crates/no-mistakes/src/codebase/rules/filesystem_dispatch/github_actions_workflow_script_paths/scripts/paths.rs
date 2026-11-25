use super::super::options::{matches_script_dir, prefixes_script_dir};
use super::{ScriptHit, ScriptHitKind, WorkingDir};
use crate::codebase::rules::github_actions_download_integrity::words::{Command, Word};

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

pub(super) fn apply_cd(current: &WorkingDir, command: &Command, at: usize) -> WorkingDir {
    let Some(arg) = command.words.get(at + 1) else {
        return WorkingDir::OutsideRepo;
    };
    if is_dynamic_word(arg, command.dynamic) {
        return WorkingDir::Dynamic(arg.text.clone());
    }
    let raw = arg.text.trim();
    if raw.is_empty() || raw == "-" || raw.starts_with('/') || raw.starts_with('~') {
        return WorkingDir::OutsideRepo;
    }
    match current {
        WorkingDir::Known(base) => {
            resolve_rel_path(base, raw).map_or(WorkingDir::OutsideRepo, WorkingDir::Known)
        }
        WorkingDir::Dynamic(raw_dir) => WorkingDir::Dynamic(raw_dir.clone()),
        WorkingDir::OutsideRepo => WorkingDir::OutsideRepo,
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

fn dynamic_may_target_script_dir(raw: &str, dir: &WorkingDir, script_dirs: &[String]) -> bool {
    if raw.starts_with('/') || raw.starts_with('~') {
        return false;
    }
    match dir {
        WorkingDir::OutsideRepo => false,
        WorkingDir::Dynamic(_) => true,
        WorkingDir::Known(base) => {
            if !base.is_empty() && prefixes_script_dir(base, script_dirs) {
                return true;
            }
            let prefix = raw
                .split(['$', '{'])
                .next()
                .unwrap_or("")
                .trim_end_matches('/');
            if prefix.is_empty() || prefix == "." {
                return true;
            }
            resolve_rel_path(base, prefix).is_some_and(|resolved| {
                prefixes_script_dir(&resolved, script_dirs)
                    || script_dirs
                        .iter()
                        .any(|dir| dir.starts_with(&format!("{resolved}/")))
            })
        }
    }
}

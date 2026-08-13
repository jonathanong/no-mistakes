use super::{join_relative, normalize_repo_relative, scan_tokens, static_tokens};

mod comments;
mod failures;
mod shape;

use comments::strip_static_comments;
pub(crate) use failures::{
    shell_body_before_static_failure, shell_body_has_static_failure,
    shell_body_has_static_failure_with_initial, shell_body_has_static_pipeline_failure,
    shell_body_is_statically_successful,
};
use shape::{effective_tokens, is_unsupported_control_command};

pub(crate) use shape::shell_body_has_safe_static_shape;

pub(super) fn scan_shell_body_for_typechecked_projects(
    script: &str,
    initial_cwd: &str,
    failure_enforced: bool,
) -> Vec<String> {
    let script = strip_static_comments(script);
    if !shape::shell_body_has_safe_static_shape(&script) {
        return Vec::new();
    }
    let mut cwd = normalize_repo_relative(initial_cwd);
    let mut projects = Vec::new();
    let groups = script
        .split(['\n', ';'])
        .filter(|segment| !segment.trim().is_empty())
        .collect::<Vec<_>>();
    for (group_index, group) in groups.iter().enumerate() {
        let segments = group
            .split("&&")
            .filter(|segment| !segment.trim().is_empty())
            .collect::<Vec<_>>();
        let final_group = group_index + 1 == groups.len();
        if segments.len() > 1 && !final_group {
            return Vec::new();
        }
        let segment_count = segments.len();
        for (segment_index, segment) in segments.into_iter().enumerate() {
            let Some(tokens) = static_tokens(segment) else {
                if segment_index + 1 < segment_count {
                    break;
                }
                continue;
            };
            let Some(tokens) = effective_tokens(&tokens) else {
                return Vec::new();
            };
            let first = tokens
                .first()
                .expect("a nonblank static shell segment has at least one token");
            if is_unsupported_control_command(tokens) {
                return Vec::new();
            }
            if first == "cd" {
                cwd = (tokens.len() == 2)
                    .then(|| {
                        tokens.get(1).and_then(|path| {
                            cwd.as_ref().and_then(|base| join_relative(base, path))
                        })
                    })
                    .flatten();
                continue;
            }
            let Some(base) = cwd.as_deref() else {
                continue;
            };
            if failure_enforced || final_group {
                projects.extend(scan_tokens(tokens, base));
            }
        }
    }
    projects.sort();
    projects.dedup();
    projects
}

/// Parse the explicit local `bash|sh ... -c <literal>` shape. Unlike Actions,
/// local shells start without failure propagation unless `-e`/`errexit` is
/// present; the scanner can still credit a final `tsc` command.
pub(super) fn local_shell_command(argv: &[String]) -> Option<(&str, bool)> {
    if !matches!(argv.first()?.as_str(), "bash" | "sh") {
        return None;
    }
    let mut failure_enforced = false;
    let mut index = 1;
    while let Some(argument) = argv.get(index) {
        if argument == "-c" {
            if index + 2 != argv.len() {
                return None;
            }
            return Some((argv.get(index + 1)?.as_str(), failure_enforced));
        }
        if argument == "-o" || argument == "+o" {
            let option = argv.get(index + 1)?;
            if option != "errexit" {
                return None;
            }
            failure_enforced = argument == "-o";
            index += 2;
            continue;
        }
        let (prefix, options) = argument
            .strip_prefix('-')
            .map(|options| ("-", options))
            .or_else(|| argument.strip_prefix('+').map(|options| ("+", options)))?;
        if options.is_empty() || !options.chars().all(|option| option.is_ascii_alphabetic()) {
            return None;
        }
        if options.contains('n') || options.contains('D') {
            return None;
        }
        if options.contains('c') {
            if prefix != "-" || options.matches('c').count() != 1 || index + 2 != argv.len() {
                return None;
            }
            failure_enforced |= options.contains('e');
            return Some((argv.get(index + 1)?.as_str(), failure_enforced));
        }
        if options.contains('e') {
            failure_enforced = prefix == "-";
        }
        index += 1;
    }
    None
}

use super::paths::{self, unanalyzable};
use super::{ScanState, ScriptHit, WorkingDir, MAX_NESTED_DEPTH};
use crate::codebase::rules::filesystem_dispatch::github_actions_workflow_script_paths::extract::{
    self, ScriptArg,
};
use crate::codebase::rules::github_actions_download_integrity::words::{self, Command};

pub(super) fn inspect_command(
    name: &str,
    command: &Command,
    at: usize,
    dir: &WorkingDir,
    state: &ScanState<'_>,
    hits: &mut Vec<ScriptHit>,
) {
    let cmd_word = &command.words[at];
    let path_cmd = cmd_word.text.contains('/') && !cmd_word.text.contains("://");
    if path_cmd {
        paths::check_candidate(
            cmd_word,
            command.dynamic,
            dir,
            state.script_dirs,
            state.line_offset,
            hits,
        );
    }
    let looked_up = if path_cmd {
        words::basename(&cmd_word.text)
    } else {
        name.to_string()
    };
    match extract::interpreter_script_arg(&looked_up, &command.words, at + 1) {
        Some(ScriptArg::Path(target)) => paths::check_candidate(
            target,
            command.dynamic,
            dir,
            state.script_dirs,
            state.line_offset,
            hits,
        ),
        Some(ScriptArg::DirectoryFlag(flag)) => hits.push(unanalyzable(
            flag.line + state.line_offset,
            flag.text.clone(),
            format!(
                "runs `{name}` with `{}`, which changes the working directory, so the script path cannot be proven against workflow path filters. Invoke the script from the repository root or a literal working-directory.",
                flag.text
            ),
        )),
        None => {}
    }
}

pub(super) fn try_shell_c(
    name: &str,
    command: &Command,
    at: usize,
    dir: &WorkingDir,
    state: &ScanState<'_>,
    hits: &mut Vec<ScriptHit>,
) -> bool {
    if !matches!(name, "bash" | "sh" | "zsh" | "dash" | "ash" | "ksh") {
        return false;
    }
    let Some(operand) = super::super::shell_cmd::shell_c_operand(&command.words, at + 1) else {
        return false;
    };
    let op_line = operand.line + state.line_offset;
    if paths::is_dynamic_word(operand, command.dynamic) {
        hits.push(unanalyzable(
            op_line,
            operand.text.clone(),
            format!("passes a dynamic script to `{name} -c`, which cannot be statically checked against workflow path filters. Run the script directly with a literal path."),
        ));
    } else {
        descend(&operand.text, dir.clone(), op_line, state, hits);
    }
    true
}

pub(super) fn scan_substitutions(
    substitutions: &[(usize, usize, String)],
    timeline: &[(usize, WorkingDir)],
    state: &ScanState<'_>,
    hits: &mut Vec<ScriptHit>,
) {
    for (sub_line, start, body) in substitutions {
        let active = timeline
            .iter()
            .rev()
            .find(|(anchor, _)| *anchor < *start)
            .map(|(_, dir)| dir.clone())
            .unwrap_or_else(|| timeline[0].1.clone());
        descend(body, active, state.line_offset + sub_line, state, hits);
    }
}

fn descend(
    body: &str,
    dir: WorkingDir,
    line: usize,
    state: &ScanState<'_>,
    hits: &mut Vec<ScriptHit>,
) {
    if state.depth >= MAX_NESTED_DEPTH {
        hits.push(unanalyzable(
            line,
            String::new(),
            "nests shell commands more than 4 levels deep, so repository script paths could not be checked against workflow path filters. Flatten the nested shell command.".to_string(),
        ));
        return;
    }
    let nested = ScanState {
        script_dirs: state.script_dirs,
        line_offset: line.saturating_sub(1),
        depth: state.depth + 1,
    };
    super::scan_nested(body, dir, &nested, hits);
}

mod paths;

pub(super) use paths::resolve_rel_path;

use crate::codebase::rules::github_actions_download_integrity::words::{self, Command};
use paths::{apply_cd, check_candidate, is_dynamic_word, unanalyzable};

const MAX_NESTED_DEPTH: usize = 4;

#[derive(Clone, Debug)]
pub(super) enum WorkingDir {
    Known(String),
    OutsideRepo,
    Dynamic(String),
}

#[derive(Debug)]
pub(super) struct ScriptHit {
    pub(super) line: usize,
    pub(super) kind: ScriptHitKind,
}

#[derive(Debug)]
pub(super) enum ScriptHitKind {
    Invocation { path: String, token: String },
    Unanalyzable { token: String, detail: String },
}

impl ScriptHit {
    pub(super) fn token(&self) -> Option<&str> {
        match &self.kind {
            ScriptHitKind::Invocation { token, .. } => Some(token.as_str()),
            ScriptHitKind::Unanalyzable { token, .. } => (!token.is_empty()).then_some(token),
        }
    }
}

struct ScanState<'a> {
    script_dirs: &'a [String],
    line_offset: usize,
    depth: usize,
}

pub(super) fn scan_script(
    script: &str,
    initial_dir: WorkingDir,
    script_dirs: &[String],
) -> Vec<ScriptHit> {
    let mut hits = Vec::new();
    let state = ScanState {
        script_dirs,
        line_offset: 0,
        depth: 0,
    };
    scan_nested(script, initial_dir, &state, &mut hits);
    hits.sort_by_key(|hit| hit.line);
    hits
}

fn scan_nested(
    script: &str,
    mut dir: WorkingDir,
    state: &ScanState<'_>,
    hits: &mut Vec<ScriptHit>,
) {
    let parsed = words::parse(script);
    if let Some(reason) = syntax_abstention(&parsed) {
        hits.push(unanalyzable(
            1 + state.line_offset,
            String::new(),
            format!("contains {reason}, so invoked repository scripts could not be checked against workflow path filters. Fix the shell syntax in this step."),
        ));
        return;
    }
    let mut dir_timeline = vec![(0usize, dir.clone())];
    for command in &parsed.commands {
        let Some((name, at)) = words::command_at(&command.words) else {
            continue;
        };
        let cmd_word = &command.words[at];
        let cmd_line = cmd_word.line + state.line_offset;
        if name == "cd" {
            dir = apply_cd(&dir, command, at);
            dir_timeline.push((cmd_word.line, dir.clone()));
            continue;
        }
        if is_dynamic_word(cmd_word, command.dynamic) {
            hits.push(unanalyzable(
                cmd_line,
                cmd_word.text.clone(),
                format!("invokes dynamic command `{}`, which cannot be statically checked against workflow path filters. Call the script or command with a literal path.", cmd_word.text),
            ));
            continue;
        }
        if name == "eval" {
            hits.push(unanalyzable(
                cmd_line,
                "eval".to_string(),
                "invokes `eval`, which cannot be statically checked against workflow path filters. Replace `eval` with a direct script invocation.".to_string(),
            ));
            continue;
        }
        if try_shell_c(&name, command, at, &dir, state, hits) {
            continue;
        }
        inspect_command(&name, command, at, &dir, state, hits);
    }
    if state.depth < MAX_NESTED_DEPTH {
        for (sub_line, body) in &parsed.substitutions {
            let active_dir = dir_timeline
                .iter()
                .rev()
                .find(|(line, _)| *line < *sub_line)
                .map(|(_, dir)| dir.clone())
                .unwrap_or_else(|| dir_timeline[0].1.clone());
            let nested = ScanState {
                script_dirs: state.script_dirs,
                line_offset: state.line_offset + sub_line.saturating_sub(1),
                depth: state.depth + 1,
            };
            scan_nested(body, active_dir, &nested, hits);
        }
    }
}

fn inspect_command(
    name: &str,
    command: &Command,
    at: usize,
    dir: &WorkingDir,
    state: &ScanState<'_>,
    hits: &mut Vec<ScriptHit>,
) {
    let cmd_word = &command.words[at];
    if cmd_word.text.contains('/') && !cmd_word.text.contains("://") {
        check_candidate(
            cmd_word,
            command.dynamic,
            dir,
            state.script_dirs,
            state.line_offset,
            hits,
        );
        return;
    }
    if let Some(target) = super::extract::interpreter_script_arg(name, &command.words, at + 1) {
        check_candidate(
            target,
            command.dynamic,
            dir,
            state.script_dirs,
            state.line_offset,
            hits,
        );
    }
}

fn try_shell_c(
    name: &str,
    command: &Command,
    at: usize,
    dir: &WorkingDir,
    state: &ScanState<'_>,
    hits: &mut Vec<ScriptHit>,
) -> bool {
    if !matches!(name, "bash" | "sh" | "zsh") {
        return false;
    }
    let Some(operand) = super::extract::shell_c_operand(&command.words, at + 1) else {
        return false;
    };
    let op_line = operand.line + state.line_offset;
    if is_dynamic_word(operand, command.dynamic) {
        hits.push(unanalyzable(
            op_line,
            operand.text.clone(),
            format!("passes a dynamic script to `{name} -c`, which cannot be statically checked against workflow path filters. Run the script directly with a literal path."),
        ));
    } else if state.depth < MAX_NESTED_DEPTH {
        let nested = ScanState {
            script_dirs: state.script_dirs,
            line_offset: op_line.saturating_sub(1),
            depth: state.depth + 1,
        };
        scan_nested(&operand.text, dir.clone(), &nested, hits);
    }
    true
}

fn syntax_abstention(parsed: &words::Parse) -> Option<&'static str> {
    if parsed.unterminated_heredoc {
        return Some("an unterminated here-document");
    }
    if parsed.unterminated_quote {
        return Some("an unterminated quote");
    }
    parsed
        .unterminated_substitution
        .then_some("an unterminated shell expansion")
}

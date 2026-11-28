mod cd;
mod paths;
mod visit;

pub(super) use paths::resolve_rel_path;

use crate::codebase::rules::github_actions_download_integrity::words;
use cd::apply_cd;
use paths::{is_dynamic_word, unanalyzable};

const MAX_NESTED_DEPTH: usize = 4;

#[derive(Clone, Debug, Eq, PartialEq)]
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

pub(super) fn scan_run(
    script: &str,
    shell: Option<&str>,
    initial_dir: WorkingDir,
    script_dirs: &[String],
) -> Vec<ScriptHit> {
    if let Some(name) = shell.map(shell_base).filter(|name| !known_shell(name)) {
        return vec![unanalyzable(
            1,
            name.to_string(),
            format!(
                "runs with shell `{name}`, which this check cannot parse, so repository scripts in the step were not checked against workflow path filters. Use a shell this check parses."
            ),
        )];
    }
    scan_script(script, initial_dir, script_dirs)
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
    let mut previous = None;
    let mut dir_timeline = vec![(0usize, dir.clone())];
    let mut depth = 0i32;
    let mut conditional_cd = false;
    for command in &parsed.commands {
        let (opens, closes) = control_depth(&command.words);
        depth += opens as i32;
        let Some((name, at)) = words::command_at(&command.words) else {
            depth = (depth - closes as i32).max(0);
            continue;
        };
        let cmd_word = &command.words[at];
        let cmd_line = cmd_word.line + state.line_offset;
        if name == "cd" {
            let next = apply_cd(&dir, previous.as_ref(), command, at);
            if depth > 0 {
                conditional_cd = true;
                dir = WorkingDir::Dynamic("conditional cd".to_string());
            } else {
                conditional_cd = false;
                previous = Some(dir);
                dir = next;
            }
            dir_timeline.push((command.anchor, dir.clone()));
            depth = (depth - closes as i32).max(0);
            continue;
        }
        if closes > 0 && conditional_cd {
            dir = WorkingDir::Dynamic("conditional cd".to_string());
            dir_timeline.push((command.anchor, dir.clone()));
        }
        depth = (depth - closes as i32).max(0);
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
        if visit::try_shell_c(&name, command, at, &dir, state, hits) {
            continue;
        }
        visit::inspect_command(&name, command, at, &dir, state, hits);
    }
    visit::scan_substitutions(&parsed.substitutions, &dir_timeline, state, hits);
}

fn control_depth(words: &[words::Word]) -> (usize, usize) {
    let mut opens = 0;
    let mut closes = 0;
    for word in words {
        match word.text.as_str() {
            "if" | "while" | "until" | "for" | "case" => opens += 1,
            "fi" | "done" | "esac" => closes += 1,
            _ => {}
        }
    }
    (opens, closes)
}

fn shell_base(shell: &str) -> &str {
    let token = shell.split_whitespace().next().unwrap_or(shell);
    token.rsplit(['/', '\\']).next().unwrap_or(token)
}

fn known_shell(name: &str) -> bool {
    matches!(
        name,
        "bash" | "sh" | "zsh" | "dash" | "ash" | "ksh" | "pwsh" | "powershell"
    )
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

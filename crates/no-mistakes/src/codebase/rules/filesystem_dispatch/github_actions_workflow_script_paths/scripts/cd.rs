use super::paths::{is_dynamic_word, resolve_rel_path};
use super::WorkingDir;
use crate::codebase::rules::github_actions_download_integrity::words::{Command, Word};

pub(super) fn apply_cd(
    current: &WorkingDir,
    previous: Option<&WorkingDir>,
    command: &Command,
    at: usize,
) -> WorkingDir {
    match cd_operand(command, at) {
        CdOperand::Home => WorkingDir::OutsideRepo,
        CdOperand::Previous => previous
            .cloned()
            .unwrap_or_else(|| WorkingDir::Dynamic("-".to_string())),
        CdOperand::Uncertain(token) => WorkingDir::Dynamic(token),
        CdOperand::Path(raw) => resolve_cd_path(current, &raw),
    }
}

enum CdOperand {
    Home,
    Previous,
    Uncertain(String),
    Path(String),
}

fn cd_operand(command: &Command, at: usize) -> CdOperand {
    let mut idx = at + 1;
    while let Some(word) = command.words.get(idx) {
        let text = word.text.as_str();
        if text == "--" {
            return command
                .words
                .get(idx + 1)
                .map(|next| classify_cd_word(next, command.dynamic))
                .unwrap_or(CdOperand::Home);
        }
        if is_cd_physical_flag(text) {
            idx += 1;
            continue;
        }
        return classify_cd_word(word, command.dynamic);
    }
    CdOperand::Home
}

fn classify_cd_word(word: &Word, command_dynamic: bool) -> CdOperand {
    if is_dynamic_word(word, command_dynamic) {
        return CdOperand::Uncertain(word.text.clone());
    }
    let raw = word.text.trim();
    if raw.is_empty() {
        return CdOperand::Home;
    }
    if raw == "-" {
        return CdOperand::Previous;
    }
    if raw.starts_with('-') {
        return CdOperand::Uncertain(raw.to_string());
    }
    CdOperand::Path(raw.to_string())
}

fn is_cd_physical_flag(text: &str) -> bool {
    let Some(cluster) = text
        .strip_prefix('-')
        .filter(|body| !body.is_empty() && !body.starts_with('-'))
    else {
        return false;
    };
    cluster
        .chars()
        .all(|ch| matches!(ch, 'L' | 'P' | 'e' | '@'))
}

fn resolve_cd_path(current: &WorkingDir, raw: &str) -> WorkingDir {
    if raw.starts_with('/') || raw.starts_with('~') {
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

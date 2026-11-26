use super::{scan_nested_body, unanalyzable, Hit};
use crate::codebase::rules::github_actions_download_integrity::words::{Command, Word};
use crate::codebase::rules::github_actions_run_command_policy::{
    matcher, options::CompiledOptions,
};

pub(super) fn try_scan_shell_c(
    name: &str,
    command: &Command,
    at: usize,
    line_offset: usize,
    depth: usize,
    compiled: &CompiledOptions,
    hits: &mut Vec<Hit>,
) -> bool {
    if !matches!(name, "bash" | "sh" | "zsh") {
        return false;
    }
    let Some(operand) = shell_c_operand(&command.words, at + 1) else {
        return false;
    };
    let op_line = operand.line + line_offset;
    if matcher::is_dynamic_word(operand, command.dynamic) {
        let detail = format!(
            "passes a dynamic script to `{name} -c`, which cannot be statically checked against configured command policies. Inline the literal command or run it directly."
        );
        hits.push(unanalyzable(op_line, operand.text.clone(), detail));
    } else {
        scan_nested_body(
            &operand.text,
            op_line,
            "a literal shell script",
            depth,
            compiled,
            hits,
        );
    }
    true
}

fn shell_c_operand(words: &[Word], mut index: usize) -> Option<&Word> {
    while let Some(word) = words.get(index) {
        let text = word.text.as_str();
        if text == "-c" || text == "--command" {
            return words.get(index + 1);
        }
        if let Some(cluster) = text.strip_prefix('-').filter(|body| !body.starts_with('-')) {
            if let Some((_, after)) = cluster.split_once('c') {
                return after.is_empty().then(|| words.get(index + 1)).flatten();
            }
        }
        if !text.starts_with('-') {
            return None;
        }
        index += 1 + usize::from(matches!(text, "-o" | "-O" | "--rcfile"));
    }
    None
}

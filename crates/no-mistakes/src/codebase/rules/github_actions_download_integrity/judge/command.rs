use super::super::{assign, words};
use std::cell::OnceCell;

/// Prove a declared downloader behind a command alias, never recover URL values.
pub(super) fn recover_downloader(
    command: &mut words::Command,
    script: &str,
    helpers: &[String],
    binding: &OnceCell<Option<(String, String)>>,
) -> Option<(String, usize)> {
    let identity = binding
        .get_or_init(|| assign::single_literal_binding(script))
        .as_ref()?;
    loop {
        let (name, at) = words::command_at(&command.words)?;
        if words::is_downloader(&name, helpers) {
            return Some((name, at));
        }
        let original = &command.words[at];
        if !original.text.contains('$') {
            return None;
        }
        if original.quoted_expansion.is_some()
            && (identity.1.is_empty() || identity.1.contains(char::is_whitespace))
        {
            return None;
        }
        let original_line = original.line;
        let replaced = assign::substitute_command(&original.text, identity);
        if replaced == original.text {
            return None;
        }
        let mut parsed = words::parse(&replaced);
        if parsed.commands.is_empty() {
            if replaced.trim().is_empty() {
                // Max elides this empty literal wrapper before seeing the command.
                command.words.remove(at);
                continue;
            }
            return None;
        }
        if parsed.unterminated_heredoc || parsed.commands.len() != 1 {
            return None;
        }
        let mut prefix = parsed.commands.pop()?.words;
        for word in &mut prefix {
            word.line = original_line;
        }
        command.words.splice(at..at + 1, prefix);
    }
}

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
        let original_line = original.line;
        let replaced = assign::substitute_command(&original.text, identity);
        if replaced == original.text {
            return None;
        }
        let mut parsed = words::parse(&replaced);
        if parsed.unterminated_heredoc || parsed.commands.is_empty() {
            return None;
        }
        if parsed.commands.len() > 1 {
            // Max can substitute a literal prefix containing separators. Keep
            // only downloader identity proof, reporting the whole alias shape.
            let mut suffix = Some(command.words.split_off(at + 1));
            command.words.pop();
            let mut leading = Some(std::mem::take(&mut command.words));
            let last = parsed.commands.len() - 1;
            for (index, mut candidate) in parsed.commands.into_iter().enumerate() {
                for word in &mut candidate.words {
                    word.line = original_line;
                }
                if index == 0 {
                    let mut words = leading.take().unwrap_or_default();
                    words.extend(candidate.words);
                    candidate.words = words;
                }
                if index == last {
                    candidate.words.extend(suffix.take().unwrap_or_default());
                }
                if let Some((name, slot)) =
                    recover_downloader(&mut candidate, script, helpers, binding)
                {
                    if super::url_words(&name, &candidate.words[slot + 1..])
                        .iter()
                        .any(|(_, url)| super::dynamic(url) || super::url::http_url(url).is_some())
                    {
                        command.words = candidate.words;
                        return Some((name, slot));
                    }
                }
            }
            return None;
        }
        let mut prefix = parsed.commands.pop()?.words;
        for word in &mut prefix {
            word.line = original_line;
        }
        command.words.splice(at..at + 1, prefix);
    }
}

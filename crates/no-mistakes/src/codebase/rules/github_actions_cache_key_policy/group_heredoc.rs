use super::super::super::output_syntax::{heredoc_end, heredoc_start, inline_assign};
use super::super::{Event, Write};
use super::emit::Pending;
use super::stdout;
use crate::fx::FxHashSet;

pub(super) fn take(
    lines: &[&str],
    index: usize,
    pending: &mut Vec<Pending>,
    outputs: &FxHashSet<String>,
) -> Option<usize> {
    let line = lines.get(index)?.trim();
    let (delim, strip, quoted) = heredoc_start(line)?;
    let binding = stdout::stdout_binding(line, outputs);
    let mut cursor = index + 1;
    while cursor < lines.len() && !heredoc_end(lines[cursor], &delim, strip) {
        if let Some((name, value)) = inline_assign(lines[cursor]) {
            let write = Write {
                name,
                value,
                expand: !quoted,
            };
            match binding {
                stdout::Stdout::Prove => pending.push(Pending::Event(Event::Write(write))),
                stdout::Stdout::Inherit => pending.push(Pending::Pass(write)),
                stdout::Stdout::Leave => {}
            }
        }
        cursor += 1;
    }
    if cursor < lines.len() {
        cursor += 1;
    }
    Some(cursor)
}

use super::super::super::output_syntax::inline_assign;
use super::super::{Event, Write};
use super::emit::Pending;
use super::stdout;
use crate::codebase::shell_heredoc::{line_opener, trimmed_body_lines};
use crate::fx::FxHashSet;

pub(super) fn take(
    lines: &[&str],
    index: usize,
    pending: &mut Vec<Pending>,
    outputs: &FxHashSet<String>,
) -> Option<usize> {
    let line = lines.get(index)?.trim();
    let (document, quoted) = line_opener(line)?;
    let binding = stdout::stdout_binding(line, outputs);
    let (body, consumed) = trimmed_body_lines(&lines[index + 1..], &document);
    for line in body {
        if let Some((name, value)) = inline_assign(line) {
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
    }
    Some(index + 1 + consumed)
}

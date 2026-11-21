use super::Event;
use crate::fx::FxHashSet;

#[path = "group_body.rs"]
mod body;
#[path = "group_emit.rs"]
mod emit;
#[path = "group_span.rs"]
mod span;
#[path = "group_stdout.rs"]
mod stdout;

pub(super) fn consume(
    lines: &[&str],
    index: usize,
    events: &mut Vec<Event>,
    outputs: &mut FxHashSet<String>,
) -> Option<usize> {
    span::scan(lines, index, events, outputs)
}

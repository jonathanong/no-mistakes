use super::super::super::lex::{identifier, Word};
use super::super::state::CollectedWrites;
use super::WriteSets;
use crate::fx::FxHashSet;

pub(super) enum Scope<'a> {
    Repeat,
    Group {
        isolated: bool,
        function: Option<(&'a str, bool)>,
    },
}

pub(super) fn record_global_writes(
    summary: &mut WriteSets,
    stack: &[Writes<'_>],
    cmd: &CollectedWrites,
    (index, pos, dynamic): (usize, usize, bool),
) {
    for name in &cmd.touches {
        let entry = summary.range_touches.entry(name.clone()).or_default();
        entry.push(index);
    }
    if in_subshell(stack) {
        return;
    }
    if cmd.dynamic_unset {
        summary.dynamic_unset_positions.push(pos);
    }
    for name in &cmd.empties {
        if !is_local_in_enclosing_function(stack, name) {
            summary
                .empty_positions
                .entry(name.clone())
                .or_default()
                .push(pos);
        }
    }
    for lower in &cmd.ci_empties {
        summary
            .ci_empty_positions
            .entry(lower.clone())
            .or_default()
            .push(pos);
    }
    for name in &cmd.touches {
        if !is_local_in_enclosing_function(stack, name) {
            if dynamic {
                summary.dynamic_touches.insert(name.clone());
            }
            summary.touches.entry(name.clone()).or_default().push(index);
        }
    }
}

pub(super) fn extend_scope_writes(stack: &mut [Writes<'_>], cmd: CollectedWrites) {
    let touches: Vec<String> = cmd
        .touches
        .into_iter()
        .filter(|name| !is_local_in_enclosing_function(stack, name))
        .collect();
    let empties: Vec<String> = cmd
        .empties
        .into_iter()
        .filter(|name| !is_local_in_enclosing_function(stack, name))
        .collect();
    if let Some(scope) = stack.last_mut() {
        scope.touches.extend(touches);
        scope.empties.extend(empties);
        scope.ci_empties.extend(cmd.ci_empties);
        scope.dynamic_unset |= cmd.dynamic_unset;
    }
}

pub(super) fn finish_scope(
    summary: &mut WriteSets,
    writes: Writes<'_>,
    parent: Option<&mut Writes<'_>>,
) {
    if !matches!(writes.scope, Scope::Group { isolated: true, .. }) {
        if let Some(parent) = parent {
            parent.touches.extend(writes.touches.iter().cloned());
            parent.empties.extend(writes.empties.iter().cloned());
            parent.ci_empties.extend(writes.ci_empties.iter().cloned());
            parent.locals.extend(writes.locals.iter().cloned());
            parent.dynamic_unset |= writes.dynamic_unset;
        }
    }
    if !writes.touches.is_empty() {
        summary
            .touches_by_scope
            .insert(writes.index, writes.touches);
    }
    if !writes.empties.is_empty() {
        summary
            .empties_by_scope
            .insert(writes.index, writes.empties);
    }
    if !writes.ci_empties.is_empty() {
        summary
            .ci_empties_by_scope
            .insert(writes.index, writes.ci_empties);
    }
    if !writes.locals.is_empty() {
        summary.locals_by_scope.insert(writes.index, writes.locals);
    }
    if writes.dynamic_unset {
        summary.dynamic_unset_scopes.insert(writes.index);
    }
}

fn in_subshell(stack: &[Writes<'_>]) -> bool {
    stack.iter().any(|frame| {
        matches!(
            frame.scope,
            Scope::Group {
                isolated: true,
                function: None | Some((_, false)),
            }
        )
    })
}

fn is_local_in_enclosing_function(stack: &[Writes<'_>], name: &str) -> bool {
    for frame in stack.iter().rev() {
        if frame.locals.contains(name) {
            return true;
        }
        if matches!(
            frame.scope,
            Scope::Group {
                function: Some(_),
                ..
            }
        ) {
            break;
        }
    }
    false
}

pub(super) struct Writes<'a> {
    pub index: usize,
    pub scope: Scope<'a>,
    pub touches: FxHashSet<String>,
    pub empties: FxHashSet<String>,
    pub ci_empties: FxHashSet<String>,
    pub locals: FxHashSet<String>,
    pub dynamic_unset: bool,
}

impl<'a> Writes<'a> {
    pub fn new(index: usize, scope: Scope<'a>) -> Self {
        Self {
            index,
            scope,
            touches: FxHashSet::default(),
            empties: FxHashSet::default(),
            ci_empties: FxHashSet::default(),
            locals: FxHashSet::default(),
            dynamic_unset: false,
        }
    }
}

pub(super) fn track_loop<'a>(
    index: usize,
    words: &[Word<'a>],
    stack: &mut Vec<Writes<'a>>,
    summary: &mut WriteSets,
) -> Option<&'a str> {
    let mut iter = words
        .iter()
        .filter(|word| !word.redirect)
        .map(|word| word.raw)
        .skip_while(|word| matches!(*word, "!" | "then" | "else" | "do"));
    let keyword = iter.next();
    let loop_var = match keyword {
        Some("for" | "select") => iter.next().filter(|name| identifier(name)),
        _ => None,
    };
    match keyword {
        Some("for" | "while" | "until" | "select") => {
            stack.push(Writes::new(index, Scope::Repeat));
        }
        Some("done") if matches!(stack.last().map(|w| &w.scope), Some(Scope::Repeat)) => {
            if let Some(writes) = stack.pop() {
                finish_scope(summary, writes, stack.last_mut());
            }
        }
        _ => {}
    }
    loop_var
}

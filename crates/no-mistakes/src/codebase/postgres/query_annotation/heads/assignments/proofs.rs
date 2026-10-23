use super::{Value, Write};
use crate::codebase::postgres::query_annotation::heads::{
    diagnostics::Gaps,
    syntax::{Head, Syntax},
    HeadProof, Span,
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::{FxHashMap, FxHashSet};

#[derive(Default)]
pub(in crate::codebase::postgres::query_annotation::heads) struct Proofs {
    pub calls: FxHashMap<Span, HeadProof>,
}
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Outcome {
    #[default]
    NoSql,
    Clear,
    Missing,
    Unknown,
}
impl Proofs {
    pub fn new(
        syntax: &Syntax,
        module: &TypeScriptModuleFacts,
        executors: &FxHashSet<Span>,
        gaps: &Gaps,
    ) -> Self {
        let mut proofs = Self::default();
        if syntax.assignments.declarations.is_empty() {
            return proofs;
        }
        let allowed = syntax
            .calls
            .iter()
            .filter_map(|call| {
                executors
                    .contains(&call.span)
                    .then(|| call.head.reference().map(|span| (span, call)))
                    .flatten()
            })
            .collect::<FxHashMap<_, _>>();
        let undefined = syntax
            .assignments
            .declarations
            .values()
            .chain(syntax.assignments.writes.values())
            .filter_map(|write| match write.value {
                Value::Undefined(span) => Some(span),
                _ => None,
            })
            .collect::<FxHashSet<_>>();
        let shadowed = module
            .bindings
            .iter()
            .flat_map(|binding| &binding.references)
            .filter_map(|usage| {
                let span = (usage.span.start, usage.span.end);
                undefined.contains(&span).then_some(span)
            })
            .collect::<FxHashSet<_>>();
        for binding in &module.bindings {
            let declaration = (binding.span.start, binding.span.end);
            let Some(initial) = syntax.assignments.declarations.get(&declaration) else {
                continue;
            };
            let Some(function) = initial.context.function else {
                continue;
            };
            if binding.imported || !binding.runtime || gaps.overlaps(declaration) {
                continue;
            }
            let has_tag = std::iter::once(initial)
                .chain(binding.references.iter().filter_map(|usage| {
                    syntax
                        .assignments
                        .writes
                        .get(&(usage.span.start, usage.span.end))
                }))
                .any(|write| matches!(write.value, Value::Literal(Head::Tag | Head::MissingTag)));
            let mut writes = vec![*initial];
            let mut calls = Vec::new();
            let valid = binding
                .references
                .iter()
                .filter(|usage| usage.runtime)
                .all(|usage| {
                    let span = (usage.span.start, usage.span.end);
                    if gaps.overlaps(span) || syntax.assignments.unsupported_writes.contains(&span)
                    {
                        return false;
                    }
                    if let Some(write) = syntax.assignments.writes.get(&span) {
                        writes.push(*write);
                        write.context.function == Some(function)
                    } else if let Some(call) = allowed.get(&span) {
                        calls.push((span, *call));
                        call.context.function == Some(function)
                    } else {
                        // Scalar literals cannot be mutated through aliases or reads.
                        // Tagged builders retain the strict direct-use whitelist.
                        !has_tag
                    }
                });
            if !valid {
                continue;
            }
            // Iteration-target writes run after the one-time iterable expression,
            // despite appearing before it in source. Repeated calls include the
            // loop end; one-time headers do not see these deferred writes.
            writes.sort_unstable_by_key(available_after);
            let mut outcome = Outcome::NoSql;
            let prefix = writes
                .iter()
                .map(|write| {
                    outcome = outcome.max(classify(write, &shadowed, gaps));
                    (available_after(write), outcome)
                })
                .collect::<Vec<_>>();
            for (reference, call) in calls {
                // Later writes within an enclosing loop can reach the next
                // iteration's call; unrelated later assignments cannot.
                let before = call.context.repeated.map_or(call.span.0, |span| span.1);
                let index = prefix.partition_point(|(end, _)| *end <= before);
                let outcome = index.checked_sub(1).map(|index| prefix[index].1);
                let proof = match outcome {
                    Some(Outcome::Clear) => HeadProof::Literal,
                    Some(Outcome::Missing) => HeadProof::Missing,
                    _ => HeadProof::Unproven,
                };
                proofs.calls.insert(reference, proof);
            }
        }
        proofs
    }
}
fn available_after(write: &Write) -> u32 {
    write
        .context
        .repeated
        .map_or(write.span.1, |span| span.1.max(write.span.1))
}
fn classify(write: &Write, shadowed: &FxHashSet<Span>, gaps: &Gaps) -> Outcome {
    if gaps.overlaps(write.span) {
        return Outcome::Unknown;
    }
    match write.value {
        Value::Literal(Head::Literal | Head::Tag) => Outcome::Clear,
        Value::Literal(Head::Missing | Head::MissingTag) => Outcome::Missing,
        Value::NoSql => Outcome::NoSql,
        Value::Undefined(span) if !shadowed.contains(&span) => Outcome::NoSql,
        _ => Outcome::Unknown,
    }
}

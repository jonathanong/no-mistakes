use super::*;
#[path = "escaped_guarded_identity.rs"]
mod identity;
#[path = "escaped_guarded_pushes.rs"]
mod pushes;

type Fields = std::sync::Arc<FxHashMap<String, ValueExpression>>;
#[derive(Clone, PartialEq, Eq)]
struct Identity {
    binding: (usize, String),
    root: Option<super::super::mutable_origins::RootIdentity>,
    parameter: Option<u32>,
}
pub(super) struct Summary {
    at: u32,
    ready_at: u32,
    guarded: bool,
    width: usize,
    length: Option<usize>,
    context: Option<CallableId>,
    seed: Fields,
    common: Option<FxHashMap<String, Identity>>,
}

impl ImportCollector {
    pub(in super::super::super) fn note_predeclared_array_binding(
        &mut self,
        pattern: &BindingPattern<'_>,
        kind: VariableDeclarationKind,
    ) {
        if self.collect_value_flow
            && matches!(
                kind,
                VariableDeclarationKind::Const
                    | VariableDeclarationKind::Using
                    | VariableDeclarationKind::AwaitUsing
            )
        {
            self.note_stable_array_binding(pattern);
        }
    }

    pub(in super::super::super) fn note_stable_array_binding(
        &mut self,
        pattern: &BindingPattern<'_>,
    ) {
        for name in binding_names(pattern) {
            let scope = self
                .callee_binding_scope(&name)
                .unwrap_or(self.current_lexical_scope_id());
            self.receiver_escapes
                .stable_bindings
                .entry(scope)
                .or_default()
                .insert(name);
        }
    }
    pub(super) fn note_guarded_array(
        &mut self,
        scope: usize,
        name: &str,
        at: u32,
        ready_at: u32,
        fields: &Fields,
        guarded: bool,
    ) {
        let width = fields
            .keys()
            .filter_map(|key| key.parse::<usize>().ok())
            .map(|slot| slot + 1)
            .max()
            .unwrap_or(0);
        let previous = self
            .receiver_escapes
            .guarded_summaries
            .entry(scope)
            .or_default()
            .remove(name);
        let length = if guarded {
            previous
                .as_ref()
                .and_then(|summary| summary.length)
                .filter(|old| *old == width)
        } else {
            Some(width)
        };
        let common = if guarded {
            let incoming = self.array_identities(fields);
            let mut common = previous.map_or_else(fx_map, |summary| {
                summary
                    .common
                    .unwrap_or_else(|| self.array_identities(&summary.seed))
            });
            common.retain(|slot, identity| incoming.get(slot) == Some(identity));
            Some(common)
        } else {
            None
        };
        let context = self.current_function_id();
        self.receiver_escapes
            .guarded_summaries
            .entry(scope)
            .or_default()
            .insert(
                name.to_owned(),
                Summary {
                    at,
                    ready_at,
                    guarded,
                    width,
                    length,
                    context,
                    seed: std::sync::Arc::clone(fields),
                    common,
                },
            );
    }
}

pub(super) struct Projection {
    pub(super) entries: Vec<(usize, ValueExpression, bool)>,
    pub(super) tail: Option<(usize, Vec<ValueExpression>)>,
}
/// Keep the proven prefix fixed. After the first optional push, later known
/// values form one possible open tail rather than enumerating branch lengths.
pub(super) fn entries(
    collector: &ImportCollector,
    value: &ValueExpression,
    before: u32,
) -> Option<Projection> {
    let (baseline, optional) =
        super::escaped_array_bindings::guarded_entries(collector, value, before)?;
    let summary = match value {
        ValueExpression::Named { name, scope, .. } => collector
            .receiver_escapes
            .guarded_summaries
            .get(&scope.unwrap_or(0))
            .and_then(|names| names.get(name))
            .filter(|summary| summary.at <= before && summary.guarded),
        _ => None,
    };
    let mut entries = Vec::new();
    let mut tail = Vec::new();
    let mut minimum = 0;
    for (slot, value) in baseline {
        #[cfg(feature = "test-instrumentation")]
        tail_metrics::record(0);
        if optional.is_some_and(|start| slot >= start) {
            #[cfg(feature = "test-instrumentation")]
            tail_metrics::record(1);
            tail.push(value);
        } else {
            let common = summary.is_some_and(|summary| {
                summary
                    .common
                    .as_ref()
                    .is_some_and(|common| common.contains_key(&slot.to_string()))
            });
            if common {
                minimum = minimum.max(slot + 1);
            }
            let possible = !common && summary.is_some_and(|summary| slot < summary.width);
            entries.push((slot, value, possible));
        }
    }
    Some(Projection {
        entries,
        tail: optional.map(|start| (start, tail)).or_else(|| {
            summary
                .is_some_and(|summary| summary.length.is_none())
                .then_some((minimum, Vec::new()))
        }),
    })
}

#[cfg(feature = "test-instrumentation")]
#[path = "escaped_array_tail_metrics.rs"]
mod tail_metrics;

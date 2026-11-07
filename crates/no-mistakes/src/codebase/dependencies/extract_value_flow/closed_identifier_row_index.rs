use super::super::IdentifierWrite;
use super::*;
use crate::codebase::dependencies::extract::{
    FiniteIdentifierGuard, GuardSubject, ValueBinding, ValueCall, ValueExpression,
};

pub(super) type Name<'a> = (usize, &'a str);
pub(super) type Identity<'a> = (usize, &'a str, u32);

pub(super) struct Lookup<'a> {
    pub(super) source: &'a (String, usize),
    pub(super) fields: FxHashSet<&'a str>,
}

#[cfg(test)]
#[path = "closed_identifier_row_index_tests.rs"]
mod tests;

pub(super) struct Index<'a> {
    pub(super) collector: &'a ImportCollector,
    pub(super) lookups: FxHashMap<Identity<'a>, Lookup<'a>>,
    pub(super) names: FxHashMap<CallableId, Vec<&'a str>>,
    pub(super) returns: FxHashMap<CallableId, Vec<&'a ValueExpression>>,
    pub(super) calls: FxHashMap<CallableId, Vec<&'a ValueCall>>,
    pub(super) results: FxHashMap<(u32, u32), Vec<&'a ValueBinding>>,
    pub(super) mutated: FxHashSet<Name<'a>>,
    bindings: FxHashMap<Name<'a>, Vec<&'a ValueBinding>>,
    guards: FxHashMap<Identity<'a>, Vec<&'a FiniteIdentifierGuard>>,
    writes: FxHashMap<Name<'a>, Vec<&'a IdentifierWrite>>,
}

impl<'a> Index<'a> {
    pub(super) fn new(collector: &'a ImportCollector) -> Self {
        let flow = &collector.value_flow;
        let mut this = Self {
            collector,
            lookups: FxHashMap::default(),
            names: FxHashMap::default(),
            returns: FxHashMap::default(),
            calls: FxHashMap::default(),
            results: FxHashMap::default(),
            mutated: FxHashSet::default(),
            bindings: FxHashMap::default(),
            guards: FxHashMap::default(),
            writes: FxHashMap::default(),
        };
        if let Some(names) = collector.callable_bindings.first() {
            for (name, id) in names {
                this.names.entry(*id).or_default().push(name);
            }
        }
        for bounded in flow
            .bounded_bindings
            .iter()
            .filter(|binding| binding.requires_guard)
        {
            let (Some(source), [field]) = (&bounded.source, bounded.members.as_slice()) else {
                continue;
            };
            this.lookups
                .entry((bounded.scope, bounded.name.as_str(), bounded.declared_at))
                .or_insert_with(|| Lookup {
                    source,
                    fields: FxHashSet::default(),
                })
                .fields
                .insert(field);
        }
        for binding in &flow.bindings {
            this.bindings
                .entry((binding.scope, &binding.name))
                .or_default()
                .push(binding);
            if let ValueExpression::Call(start, end) = &binding.value {
                this.results
                    .entry((*start, *end))
                    .or_default()
                    .push(binding);
            }
        }
        for bindings in this.bindings.values_mut() {
            bindings.sort_by_key(|binding| binding.declared_at);
        }
        for (owner, value) in &flow.returns {
            this.returns.entry(*owner).or_default().push(value);
        }
        for call in &flow.calls {
            if let ValueExpression::Named {
                name,
                scope: Some(0),
                ..
            } = &call.callee
            {
                if let Some(owner) = collector.callable_binding_at(0, name) {
                    this.calls.entry(owner).or_default().push(call);
                }
            }
        }
        for guard in &flow.finite_identifier_guards {
            if matches!(guard.kind, GuardKind::Lookup) {
                if let Some(GuardSubject::Binding { scope, declared_at }) = guard.subject {
                    this.guards
                        .entry((scope, guard.name.as_str(), declared_at))
                        .or_default()
                        .push(guard);
                }
            }
        }
        for write in &flow.identifier_writes {
            this.writes
                .entry((write.scope, &write.name))
                .or_default()
                .push(write);
        }
        this.mutated.extend(
            flow.registry_mutations
                .iter()
                .map(|mutation| (mutation.scope, mutation.name.as_str())),
        );
        this
    }

    pub(super) fn binding(&self, name: Name<'_>, at: u32) -> Option<&'a ValueBinding> {
        let bindings = self.bindings.get(&name)?;
        let index = bindings.partition_point(|binding| binding.declared_at <= at);
        let binding = *bindings.get(index.checked_sub(1)?)?;
        binding
            .invalidated_at
            .is_none_or(|end| at < end)
            .then_some(binding)
    }

    pub(super) fn guarded(&self, identity: Identity<'_>, owner: CallableId, offset: u32) -> bool {
        self.guards.get(&identity).is_some_and(|guards| {
            guards.iter().any(|guard| {
                guard.context == Some(owner)
                    && guard.from <= offset
                    && offset < guard.until
                    && self
                        .writes
                        .get(&(identity.0, identity.1))
                        .is_none_or(|writes| {
                            writes
                                .iter()
                                .all(|write| write.at <= guard.from || write.at >= offset)
                        })
            })
        })
    }

    pub(super) fn reads(&self, name: Name<'_>, fields: &FxHashSet<&str>) -> bool {
        self.collector
            .metadata_rows
            .named
            .get(&name.0)
            .and_then(|names| names.get(name.1))
            .is_some_and(|usage| {
                !usage.invalid
                    && usage
                        .fields
                        .iter()
                        .all(|field| fields.contains(field.as_str()))
            })
    }
}

//! Exact named-import obligations and a cached complete imported-callee census.
use super::{CallSyntax, Pending, Span};
use crate::codebase::postgres::query_annotation::heads::{bindings::Bindings, syntax::Syntax};
use crate::codebase::ts_source::module_facts::ModuleBinding;
use crate::fx::{FxHashMap, FxHashSet};

pub(super) struct Imports<'a> {
    pub names: &'a FxHashMap<usize, Option<(&'a str, &'a str)>>,
    pub calls: FxHashSet<Span>,
    memo: FxHashMap<usize, bool>,
    ambiguous: FxHashSet<(&'a str, &'a str)>,
}
impl<'a> Imports<'a> {
    pub fn new(syntax: &Syntax, names: &'a FxHashMap<usize, Option<(&'a str, &'a str)>>) -> Self {
        let mut seen = FxHashSet::default();
        let ambiguous = names
            .values()
            .flatten()
            .filter_map(|&name| (!seen.insert(name)).then_some(name))
            .collect();
        Self {
            names,
            ambiguous,
            calls: syntax
                .effect_calls
                .iter()
                .filter(|call| call.position == 0)
                .map(|call| call.callee)
                .collect(),
            memo: FxHashMap::default(),
        }
    }
    pub fn dependency(
        &mut self,
        syntax: &Syntax,
        bindings: &Bindings<'_>,
        callee: &ModuleBinding,
        call: &CallSyntax,
    ) -> Option<Pending> {
        if call.position != 0 {
            return None;
        }
        let (specifier, imported) = self.names.get(&callee.id).copied().flatten()?;
        if imported == "*"
            || imported == "default"
            || self.ambiguous.contains(&(specifier, imported))
        {
            return None;
        }
        let safe = *self.memo.entry(callee.id).or_insert_with(|| {
            let mut visits = 0;
            let safe = callee.runtime
                && callee.references.iter().all(|usage| {
                    visits += 1;
                    let span = (usage.span.start, usage.span.end);
                    usage.runtime
                        && self.calls.contains(&span)
                        && bindings
                            .owner(span)
                            .is_some_and(|owner| owner.id == callee.id)
                        && !syntax.assignments.writes.contains_key(&span)
                        && !syntax.assignments.unsupported_writes.contains(&span)
                });
            if let Some(observer) = crate::diagnostics::current() {
                observer.increment("postgres.annotation.effect_import_censuses", 1);
                observer.increment("postgres.annotation.effect_import_reference_visits", visits);
            }
            safe
        });
        safe.then(|| Pending {
            span: call.span,
            builder: call.builder,
            specifier: specifier.into(),
            imported: imported.into(),
            shape: call.shape,
            require_plain: true,
        })
    }
}

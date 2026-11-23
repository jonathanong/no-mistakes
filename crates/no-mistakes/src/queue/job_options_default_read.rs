use super::super::walk::{read_member_with, read_object};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::{FxHashMap, FxHashSet};
use crate::queue::extract_model::JobOptionEvidence;
use crate::queue::object_bindings::ObjectBindings;
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum Projection {
    DefaultMember,
    OptionKeys,
}

pub(super) fn queue_defaults(
    expr: &Expression<'_>,
    bindings: &ObjectBindings<'_>,
) -> JobOptionEvidence {
    let mut reader = OptionsReader {
        bindings,
        memo: FxHashMap::default(),
        active: FxHashSet::default(),
    };
    reader
        .project(expr, expr.span().start, Projection::DefaultMember)
        .unwrap_or_default()
}

struct OptionsReader<'a, 'b> {
    bindings: &'b ObjectBindings<'a>,
    memo: FxHashMap<(u32, Projection), Option<JobOptionEvidence>>,
    active: FxHashSet<u32>,
}

impl OptionsReader<'_, '_> {
    fn project(
        &mut self,
        expr: &Expression<'_>,
        at: u32,
        projection: Projection,
    ) -> Option<JobOptionEvidence> {
        let mut evidence = match unwrap_ts_wrappers(expr) {
            Expression::ObjectExpression(object) => match projection {
                Projection::DefaultMember => {
                    read_member_with(object, "defaultJobOptions", |spread| {
                        self.project(spread, at, Projection::DefaultMember)
                    })?
                }
                Projection::OptionKeys => read_object(object),
            },
            Expression::Identifier(id) => {
                return self.project_binding(id.name.as_str(), at, projection)
            }
            _ => return Some(JobOptionEvidence::open()),
        };
        for spread in std::mem::take(&mut evidence.spreads) {
            if let Some(spread) =
                self.project_binding(&spread.name, spread.at, Projection::OptionKeys)
            {
                evidence.absorb(&spread);
            }
        }
        Some(evidence)
    }

    fn project_binding(
        &mut self,
        name: &str,
        at: u32,
        projection: Projection,
    ) -> Option<JobOptionEvidence> {
        let Some(binding) = self.bindings.lookup(name, at) else {
            return Some(JobOptionEvidence::open());
        };
        let Some(value) = binding.value() else {
            return Some(JobOptionEvidence::open());
        };
        let at = binding.resolve_at;
        let key = (at, projection);
        if let Some(found) = self.memo.get(&key) {
            return found.clone();
        }
        if !self.active.insert(at) {
            return Some(JobOptionEvidence::open());
        }
        let found = self.project(value, at, projection);
        self.active.remove(&at);
        self.memo.insert(key, found.clone());
        found
    }
}

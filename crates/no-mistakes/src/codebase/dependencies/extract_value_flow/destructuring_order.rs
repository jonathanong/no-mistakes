use super::*;
use mutation_history::{ValueAssignedMutations, ValueMutationHistory};
use oxc_ast::ast::{AssignmentTargetMaybeDefault, AssignmentTargetProperty};
use phase_catalog::{PhaseCatalog, ReadPhases};

/// Only reads in later destructuring targets precede the ordinary completion
/// cutoff. RHS reads keep their original offsets and never enter these ranges.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EarlyAssignments {
    members: FxHashMap<String, ReadPhases>,
    dynamic: ReadPhases,
}
impl EarlyAssignments {
    #[cfg(feature = "test-instrumentation")]
    pub fn catalog_entries(&self) -> usize {
        self.dynamic.len() + self.members.values().map(ReadPhases::len).sum::<usize>()
    }

    pub(super) fn merge(&mut self, source: Self) {
        self.dynamic.merge(source.dynamic);
        for (member, ranges) in source.members {
            self.members.entry(member).or_default().merge(ranges);
        }
    }
    pub(super) fn invalidated(
        &self,
        member: &str,
        reader: &impl Fn(Option<CallableId>) -> Option<u32>,
        assignments: &ValueAssignedMutations,
    ) -> bool {
        [
            (assignments.members.get(member), self.members.get(member)),
            (assignments.dynamic.as_ref(), Some(&self.dynamic)),
        ]
        .into_iter()
        .any(|(write, ranges)| {
            let Some(offset) = write.and_then(|write| reader(write.context)) else {
                return false;
            };
            ranges.is_some_and(|ranges| ranges.invalidated(offset))
        })
    }

    fn finish(&mut self) {
        for ranges in self
            .members
            .values_mut()
            .chain(std::iter::once(&mut self.dynamic))
        {
            ranges.finish();
        }
    }
}
impl ValueMutationHistory {
    #[cfg(feature = "test-instrumentation")]
    pub fn early_phase_entries(&self) -> usize {
        self.assigned
            .as_ref()
            .and_then(|assigned| assigned.early.as_ref())
            .map_or(0, |early| early.catalog_entries())
            + self
                .members
                .values()
                .map(Self::early_phase_entries)
                .sum::<usize>()
            + self
                .dynamic
                .as_ref()
                .map_or(0, |child| child.early_phase_entries())
    }
    pub(super) fn finish_early_assignments(&mut self) {
        if let Some(early) = self
            .assigned
            .as_mut()
            .and_then(|assigned| assigned.early.as_mut())
        {
            early.finish();
        }
        for child in self
            .members
            .values_mut()
            .chain(self.dynamic.iter_mut().map(Box::as_mut))
        {
            child.finish_early_assignments();
        }
    }
}
impl ImportCollector {
    pub(in super::super) fn record_early_assignment(
        &mut self,
        scope: usize,
        name: &str,
        path: &[Option<String>],
        catalog: &std::sync::Arc<PhaseCatalog>,
        phase: u32,
    ) {
        let Some((member, path)) = path.split_last() else {
            return;
        };
        let base = name.split('.').next().expect("write has a receiver");
        let Some(mut history) = self
            .prior_member_mutations
            .get_mut(&scope)
            .and_then(|names| names.get_mut(base))
        else {
            return;
        };
        for segment in path {
            history = match segment {
                Some(member) => history.members.entry(member.clone()).or_default(),
                None => history.dynamic.get_or_insert_with(Default::default),
            };
        }
        let early = history
            .assigned
            .get_or_insert_with(Default::default)
            .early
            .get_or_insert_with(Default::default);
        let target = match member {
            Some(member) => early.members.entry(member.clone()).or_default(),
            None => &mut early.dynamic,
        };
        target.record(catalog, phase);
        #[cfg(feature = "test-instrumentation")]
        {
            self.value_flow.early_assignment_phase_writes += 1;
        }
    }
}

impl ImportCollector {
    pub(super) fn has_early_assignment_target(&self, target: &AssignmentTarget<'_>) -> bool {
        match target {
            AssignmentTarget::StaticMemberExpression(member) => self.has_early_receiver(&member.object),
            AssignmentTarget::ComputedMemberExpression(member) => self.has_early_receiver(&member.object),
            AssignmentTarget::ArrayAssignmentTarget(array) => array.elements.iter().flatten().any(|target| self.has_early_default(target))
                || array.rest.as_ref().is_some_and(|rest| self.has_early_assignment_target(&rest.target)),
            AssignmentTarget::ObjectAssignmentTarget(object) => object.properties.iter().any(|property| {
                matches!(property, AssignmentTargetProperty::AssignmentTargetPropertyProperty(property)
                    if self.has_early_default(&property.binding))
            }) || object.rest.as_ref().is_some_and(|rest| self.has_early_assignment_target(&rest.target)),
            _ => false,
        }
    }
    fn has_early_default(&self, target: &AssignmentTargetMaybeDefault<'_>) -> bool {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(default) => {
                self.has_early_assignment_target(&default.binding)
            }
            _ => target
                .as_assignment_target()
                .is_some_and(|target| self.has_early_assignment_target(target)),
        }
    }
    fn has_early_receiver(&self, object: &Expression<'_>) -> bool {
        match crate::codebase::ts_source::unwrap_ts_wrappers(object) {
            Expression::StaticMemberExpression(member) => self.has_early_receiver(&member.object),
            Expression::ComputedMemberExpression(member) => self.has_early_receiver(&member.object),
            Expression::Identifier(identifier) => {
                let name = identifier.name.as_str();
                let scope = self.callee_binding_scope(name).unwrap_or(0);
                self.prior_member_mutations
                    .get(&scope)
                    .is_some_and(|names| names.contains_key(name))
            }
            _ => false,
        }
    }
}

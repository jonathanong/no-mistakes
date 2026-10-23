use super::{
    diagnostics::Gaps,
    syntax::{Head, Syntax},
    HeadProof, Span,
};
use crate::codebase::ts_source::{facts::TsFileFacts, module_facts::ModuleBinding};
use crate::fx::{FxHashMap, FxHashSet};
pub(super) struct Bindings<'a> {
    owners: FxHashMap<Span, Option<&'a ModuleBinding>>,
    constants: &'a FxHashMap<Span, Head>,
    gaps: Option<Gaps>,
    safe_tags: FxHashSet<Span>,
    mutable: super::assignments::Proofs,
    append_safety: super::append_constants::Safety,
}
impl<'a> Bindings<'a> {
    pub fn new(
        syntax: &'a Syntax,
        ts: &'a TsFileFacts,
        executor_calls: impl Iterator<Item = Span>,
    ) -> Self {
        let candidates = || {
            syntax
                .calls
                .iter()
                .map(|call| call.head)
                .chain(syntax.constants.values().copied())
        };
        let wanted = candidates()
            .filter_map(|head| head.reference())
            .chain(syntax.append_tags.values().copied())
            .collect::<FxHashSet<_>>();
        let mut index = Self {
            owners: FxHashMap::default(),
            constants: &syntax.constants,
            gaps: None,
            safe_tags: FxHashSet::default(),
            mutable: super::assignments::Proofs::default(),
            append_safety: Default::default(),
        };
        let Some(module) = ts.module_bindings.as_deref().filter(|_| !wanted.is_empty()) else {
            return index;
        };
        index.gaps = Some(Gaps::new(module));
        let executors = executor_calls.collect::<FxHashSet<_>>();
        index.append_safety = super::append_constants::Safety::new(syntax, module, &executors);
        index.safe_tags = super::tag_constants::safe(syntax, module, executors.iter().copied());
        index.mutable = super::assignments::Proofs::new(
            syntax,
            module,
            &executors,
            index.gaps.as_ref().unwrap(),
        );
        // Index every possible owner once, retaining only relevant references.
        // A duplicate is ambiguous even if the other owner is invalid.
        for binding in &module.bindings {
            for usage in binding.references.iter().filter(|usage| usage.runtime) {
                let span = (usage.span.start, usage.span.end);
                if wanted.contains(&span) {
                    index
                        .owners
                        .entry(span)
                        .and_modify(|owner| *owner = None)
                        .or_insert(Some(binding));
                }
            }
        }
        index
    }
    pub(super) fn owner(&self, span: Span) -> Option<&ModuleBinding> {
        let owner = self.owners.get(&span).copied().flatten()?;
        let gaps = self.gaps.as_ref()?;
        (owner.runtime
            && !gaps.overlaps(span)
            && !gaps.overlaps((owner.span.start, owner.span.end)))
        .then_some(owner)
    }
    pub fn proof(&self, head: Head, call: Span) -> HeadProof {
        match head {
            Head::Reference(span) | Head::AppendReference(span) => {
                let Some(owner) = self.owner(span).filter(|owner| !owner.imported) else {
                    return HeadProof::Unproven;
                };
                let declaration = (owner.span.start, owner.span.end);
                let Some(value) = self.constants.get(&declaration).copied() else {
                    return self
                        .mutable
                        .calls
                        .get(&span)
                        .copied()
                        .unwrap_or(HeadProof::Unproven);
                };
                if matches!(head, Head::AppendReference(_))
                    || value.appended()
                    || (value.tagged() && self.append_safety.touched(declaration))
                {
                    return if self.append_safety.accepts(declaration, call) {
                        value.proof(
                            matches!(head, Head::AppendReference(_))
                                || self.append_safety.continued(declaration, call),
                        )
                    } else {
                        HeadProof::Unproven
                    };
                }
                if value.tagged()
                    && !self.safe_tags.contains(&declaration)
                    && !self.append_safety.accepts(declaration, call)
                {
                    HeadProof::Unproven
                } else {
                    value.proof(false)
                }
            }
            value => value.proof(false),
        }
    }
}

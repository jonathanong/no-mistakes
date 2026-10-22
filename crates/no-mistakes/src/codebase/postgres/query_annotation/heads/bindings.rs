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
            .collect::<FxHashSet<_>>();
        let mut index = Self {
            owners: FxHashMap::default(),
            constants: &syntax.constants,
            gaps: None,
            safe_tags: FxHashSet::default(),
        };
        let Some(module) = ts.module_bindings.as_deref().filter(|_| !wanted.is_empty()) else {
            return index;
        };
        index.gaps = Some(Gaps::new(module));
        index.safe_tags = super::tag_constants::safe(syntax, module, executor_calls);
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
    fn owner(&self, span: Span) -> Option<&ModuleBinding> {
        let owner = self.owners.get(&span).copied().flatten()?;
        let gaps = self.gaps.as_ref()?;
        (owner.runtime
            && !gaps.overlaps(span)
            && !gaps.overlaps((owner.span.start, owner.span.end)))
        .then_some(owner)
    }
    pub fn proof(&self, head: Head) -> HeadProof {
        match head {
            Head::Literal => HeadProof::Literal,
            Head::Tag => HeadProof::Literal,
            Head::Reference(span) => {
                let Some(owner) = self.owner(span).filter(|owner| !owner.imported) else {
                    return HeadProof::Unproven;
                };
                match self.constants.get(&(owner.span.start, owner.span.end)) {
                    Some(Head::Literal) => HeadProof::Literal,
                    Some(Head::Tag)
                        if self.safe_tags.contains(&(owner.span.start, owner.span.end)) =>
                    {
                        HeadProof::Literal
                    }
                    _ => HeadProof::Unproven,
                }
            }
            Head::Unproven => HeadProof::Unproven,
        }
    }
}

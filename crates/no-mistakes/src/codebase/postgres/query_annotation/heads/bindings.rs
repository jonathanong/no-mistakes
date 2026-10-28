use super::{
    diagnostics::Gaps,
    syntax::{Head, Syntax},
    Span,
};
use crate::codebase::ts_source::{facts::TsFileFacts, module_facts::ModuleBinding};
use crate::fx::{FxHashMap, FxHashSet};
mod prefix;
mod proof;
mod record_scope;
mod wanted;
pub(super) struct Bindings<'a> {
    module: Option<&'a crate::codebase::ts_source::module_facts::TypeScriptModuleFacts>,
    owners: FxHashMap<Span, Option<&'a ModuleBinding>>,
    constants: &'a FxHashMap<Span, Head>,
    gaps: Option<Gaps>,
    safe_tags: FxHashSet<Span>,
    mutable: super::assignments::Proofs,
    append_safety: super::append_constants::Safety,
    trusted_imports: FxHashSet<usize>,
    effects: super::effects::Prepared,
    trusted_roots: FxHashSet<Span>,
    returned_prefix_roots: FxHashSet<Span>,
}
impl<'a> Bindings<'a> {
    pub fn new(
        syntax: &'a Syntax,
        ts: &'a TsFileFacts,
        executor_calls: impl Iterator<Item = Span>,
        low: bool,
    ) -> Self {
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("postgres.annotation.head_binding_indexes", 1);
        }
        let mut wanted = wanted::references(syntax);
        let mut index = Self {
            module: None,
            owners: FxHashMap::default(),
            constants: &syntax.constants,
            gaps: None,
            safe_tags: FxHashSet::default(),
            mutable: super::assignments::Proofs::default(),
            append_safety: Default::default(),
            trusted_imports: Default::default(),
            effects: Default::default(),
            trusted_roots: Default::default(),
            returned_prefix_roots: Default::default(),
        };
        let Some(module) = ts.module_bindings.as_deref().filter(|_| {
            !wanted.is_empty()
                || !syntax.helper_returns.is_empty()
                || !syntax.record_returns.is_empty()
                || !syntax.finite_returns.is_empty()
                || !syntax.effect_functions.is_empty()
                || !syntax.string_constants.is_empty()
        }) else {
            return index;
        };
        // A closed captured root needs ownership for every runtime reference,
        // including uses that are not executor or append candidates.
        for binding in &module.bindings {
            if syntax
                .append_tags
                .contains_key(&(binding.span.start, binding.span.end))
            {
                wanted.extend(
                    binding
                        .references
                        .iter()
                        .filter(|usage| usage.runtime)
                        .map(|usage| (usage.span.start, usage.span.end)),
                );
            }
        }
        index.gaps = Some(Gaps::new(module));
        index.module = Some(module);
        index.trusted_imports = super::trusted_append::import_ids(module);
        // Index owners first so trusted roots use the same ambiguity and gap proof.
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
        let executors = executor_calls.collect::<FxHashSet<_>>();
        index.trusted_roots = index.trusted_roots(syntax);
        let effects = super::effects::prepare(syntax, &index, module, &index.trusted_roots);
        index.effects = effects;
        if low {
            index.returned_prefix_roots =
                super::captured_prefix::return_proofs(syntax, module, &index);
        }
        let closed = if low {
            super::captured_prefix::proofs(syntax, module, &index, &executors)
        } else {
            FxHashSet::default()
        };
        index.append_safety = super::append_constants::Safety::new(
            syntax,
            module,
            &executors,
            &index.trusted_roots,
            &index.effects,
            closed,
        );
        index.safe_tags = super::tag_constants::safe(syntax, module, executors.iter().copied());
        index.mutable = super::assignments::Proofs::new(
            syntax,
            module,
            &executors,
            index.gaps.as_ref().unwrap(),
        );
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
    pub(super) fn declaration_safe(&self, span: Span) -> bool {
        self.gaps.as_ref().is_some_and(|gaps| !gaps.overlaps(span))
    }
    pub(super) fn valid_mutable_call(&self, reference: Span) -> bool {
        self.mutable.calls.contains_key(&reference)
    }
    pub(super) fn trusted_root(&self, declaration: Span) -> bool {
        self.trusted_roots.contains(&declaration)
    }
    pub(super) fn has_trusted_roots(&self) -> bool {
        !self.trusted_roots.is_empty()
    }
    pub(super) fn allowed_effect(&self, reference: Span) -> bool {
        self.effects.allowed.contains(&reference)
    }
    pub(super) fn effects(&self) -> &super::effects::Prepared {
        &self.effects
    }
    pub(super) fn stable_condition(
        &self,
        condition: &super::loop_carried::condition_key::Condition,
        loop_span: Span,
        syntax: &Syntax,
    ) -> Option<Vec<usize>> {
        condition
            .references
            .iter()
            .map(|&reference| {
                let owner = self.owner(reference)?;
                let declaration = (owner.span.start, owner.span.end);
                // Called closures can write outside the loop's lexical span;
                // retain exact function ownership instead of assuming locality.
                let function = syntax
                    .contexts
                    .get(&reference)
                    .copied()
                    .flatten()
                    .map(|span| span.0);
                let module_mutable = syntax
                    .assignments
                    .declarations
                    .get(&declaration)
                    .is_some_and(|initial| initial.context.function.is_none());
                if owner.imported
                    || inside(loop_span, declaration)
                    || owner.references.iter().any(|usage| {
                        let span = (usage.span.start, usage.span.end);
                        usage.runtime
                            && (syntax.assignments.unsupported_writes.contains(&span)
                                || syntax.assignments.writes.get(&span).is_some_and(|write| {
                                    inside(loop_span, span)
                                        || write.context.function != function
                                        || module_mutable
                                }))
                    })
                {
                    None
                } else {
                    Some(owner.id)
                }
            })
            .collect()
    }
}
fn inside(outer: Span, inner: Span) -> bool {
    outer.0 <= inner.0 && inner.1 <= outer.1
}

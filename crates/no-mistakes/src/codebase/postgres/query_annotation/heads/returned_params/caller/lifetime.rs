//! Closed native result lifetime and imported constructor/factory identity.
use super::super::Root;
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings, syntax::Syntax, HeadProof, Span,
};
use crate::codebase::ts_source::module_facts::ModuleBinding;
use crate::fx::{FxHashMap, FxHashSet};
#[derive(Clone, Copy)]
pub(super) struct Proof {
    pub callee: usize,
    pub head: HeadProof,
}
#[derive(Default)]
pub(super) struct Memo {
    pub owners: FxHashMap<usize, Option<Proof>>,
    stable: FxHashMap<usize, bool>,
    pub owner_censuses: u64,
    pub references: u64,
    pub imports: u64,
    pub import_references: u64,
}
impl Memo {
    fn stable(&mut self, syntax: &Syntax, bindings: &Bindings<'_>, owner: &ModuleBinding) -> bool {
        if let Some(value) = self.stable.get(&owner.id) {
            return *value;
        }
        self.imports += 1;
        let valid = owner.imported
            && owner.references.iter().all(|usage| {
                self.import_references += 1;
                let span = (usage.span.start, usage.span.end);
                bindings.declaration_safe(span)
                    && !syntax.assignments.writes.contains_key(&span)
                    && !syntax.assignments.unsupported_writes.contains(&span)
            });
        self.stable.insert(owner.id, valid);
        valid
    }
    pub fn prove(
        &mut self,
        syntax: &Syntax,
        bindings: &Bindings<'_>,
        owner: &ModuleBinding,
        root: &Root,
        reads: &[(Span, Span, Span)],
    ) -> Option<Proof> {
        self.owner_censuses += 1;
        let declaration = (owner.span.start, owner.span.end);
        if owner.imported
            || !bindings.declaration_safe(declaration)
            || !bindings.declaration_safe(root.span)
            || root.span.1 <= declaration.1
        {
            return None;
        }
        let context = syntax.contexts.get(&declaration)?;
        let mut callee = None;
        for leaf in &root.leaves {
            if syntax.repeated_calls.contains(&leaf.call) {
                return None;
            }
            let actual = bindings.owner(leaf.callee)?;
            let tag = bindings.owner(leaf.tag)?;
            if callee.is_some_and(|previous| previous != actual.id)
                || !self.stable(syntax, bindings, actual)
                || !tag.imported
                || !bindings.trusted_import(tag.id)
                || !self.stable(syntax, bindings, tag)
            {
                return None;
            }
            callee = Some(actual.id);
        }
        let mut allowed = FxHashSet::default();
        for &(call, text, values) in reads {
            if !bindings.declaration_safe(call)
                || text == values
                || syntax.repeated_calls.contains(&call)
                || syntax.optional_calls.contains(&call)
            {
                return None;
            }
            allowed.insert(text);
            allowed.insert(values);
        }
        let mut seen = FxHashSet::default();
        let mut paired = 0;
        for usage in &owner.references {
            self.references += 1;
            let span = (usage.span.start, usage.span.end);
            if !usage.runtime
                || span.0 <= root.span.1
                || !seen.insert(span)
                || syntax.contexts.get(&span) != Some(context)
                || bindings
                    .owner(span)
                    .is_none_or(|actual| actual.id != owner.id)
                || syntax.assignments.writes.contains_key(&span)
                || syntax.assignments.unsupported_writes.contains(&span)
            {
                return None;
            }
            if allowed.contains(&span) {
                paired += 1;
                continue;
            }
            if syntax.repeated_append_refs.contains(&span)
                || !(syntax.discarded_appends.contains(&span)
                    || syntax.prefix_discarded_appends.contains(&span))
            {
                return None;
            }
        }
        (paired == allowed.len() && paired > 0).then_some(Proof {
            callee: callee?,
            head: root.proof,
        })
    }
    pub fn report(&self) {
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment(
                "postgres.annotation.returned_caller_owner_censuses",
                self.owner_censuses,
            );
            observer.increment(
                "postgres.annotation.returned_caller_reference_visits",
                self.references,
            );
            observer.increment(
                "postgres.annotation.returned_caller_import_censuses",
                self.imports,
            );
            observer.increment(
                "postgres.annotation.returned_caller_import_reference_visits",
                self.import_references,
            );
        }
    }
}

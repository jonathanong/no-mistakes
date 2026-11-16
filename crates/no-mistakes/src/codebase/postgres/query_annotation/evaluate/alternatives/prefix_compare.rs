use super::super::{Evaluator, Scope};
use super::arena::Arena;
use crate::fx::FxHashSet;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn compare_alternative_prefixes(
        &mut self,
        scopes: &[Scope],
        before_arena: Arena<'_>,
        precise_records: bool,
        path: &Path,
        changed: &mut FxHashSet<u64>,
    ) {
        let mut comparisons = scopes
            .iter()
            .zip(&self.scopes)
            .filter(|(before, after)| {
                !precise_records
                    && before.contains_reference()
                    && !super::values::unchanged_without_arguments(before, after)
            })
            .peekable();
        // Peek retains the first eligible scope without a second scan.
        if comparisons.peek().is_some() {
            super::super::super::progress::trace(
                "postgres.annotation.prefix_compare_arm",
                path,
                None,
                || {
                    let mut work = super::values::Walk::new();
                    for (before, after) in comparisons {
                        for (name, value) in before.reference_bindings() {
                            // An arm can drop a name the other branch still has.
                            let after = super::bindings::binding_or_unproven(after, name);
                            if super::values::unchanged(value, after) {
                                if let Some(work) = work.as_mut() {
                                    work.unchanged_binding();
                                }
                            } else {
                                super::values::changes(
                                    value,
                                    after,
                                    before_arena,
                                    super::arena::Arena {
                                        objects: &self.argument_objects,
                                        extras: &self.argument_extra_slots,
                                    },
                                    changed,
                                    &self.definite_deleted_argument_slots,
                                    &mut work,
                                );
                            }
                        }
                    }
                },
            );
        }
    }
}

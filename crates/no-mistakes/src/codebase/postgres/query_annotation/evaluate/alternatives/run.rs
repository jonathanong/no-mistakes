use super::super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::Path;
impl<F: Fn(&str, &Path) -> Option<std::path::PathBuf>> Evaluator<'_, F> {
    pub(in super::super) fn alternatives(
        &mut self,
        arms: &[Expr],
        path: &Path,
        env: &Environment,
        depth: u8,
        generic: bool,
    ) -> Value {
        let precise = self.files[path].facts.precise_records;
        let base = self.scopes.checkpoint();
        let identities = self.next_builder;
        self.begin_alternative_module_snapshot(base);
        let mut joined = super::delta::Join::default();
        let mut returned = Vec::with_capacity(arms.len());
        for arm in arms {
            let checkpoint = self.begin_transaction();
            returned.push(self.expr(arm, path, env, depth, generic));
            self.revoke_changed_freshness(checkpoint);
            self.collect_arm_generation(checkpoint, &returned);
            joined.add(self.finish_transaction(checkpoint), precise);
            self.assert_checkpoint_indexes();
        }
        joined.apply(self);
        if !precise {
            self.taint_changed_bindings(&joined.changed_builders);
        }
        self.assert_checkpoint_indexes();
        joined.release_pins(self);
        let mut remembered = joined.remembered_roots(self);
        remembered.extend(self.finish_alternative_module_snapshot());
        self.collect_final_generation(base, identities, &remembered, &returned);
        self.assert_checkpoint_indexes();
        if self.transactions.frames.is_empty() && self.active_module_initials.is_empty() {
            self.scopes.clear_allocation_log();
        }
        super::super::concat::alternative_result(returned, precise)
    }
}

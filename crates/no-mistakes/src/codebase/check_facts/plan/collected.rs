use super::CheckFactPlan;

impl CheckFactPlan {
    /// TS graph fact shapes guaranteed for every file collected with this
    /// plan. Several check-only flags populate the canonical `TsFileFacts`
    /// fields too; recording only `graph` would make a strict prepared graph
    /// reject facts that were already collected in the same parse pass.
    pub(in crate::codebase::check_facts) fn collected_ts_plan(
        &self,
    ) -> crate::codebase::ts_source::facts::TsFactPlan {
        let mut plan = self.graph;
        plan.imports |= self.query_annotation
            || self.query_annotation_heads
            || self.embedded_sql
            || !self.embedded_sql_options.is_empty();
        if self.imports {
            plan.include(crate::codebase::ts_source::facts::TsFactPlan::imports());
        }
        plan.module_bindings |= self.query_annotation_heads;
        plan.symbols |= self.symbols;
        plan.source |= self.source || self.raw_source;
        plan.react |= self.react;
        plan.queue_project |= self.queue;
        plan
    }
}

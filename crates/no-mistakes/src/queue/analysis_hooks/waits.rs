use crate::codebase::ts_source::facts::TsFactContext;
use crate::queue::extract::{
    needs_enqueue_scoping, needs_job_id_scoping, needs_rate_limit_scoping, needs_scheduler_scoping,
    needs_wait_scoping, waits_in_scope,
};
use std::path::Path;

pub(super) fn semantic_demand(
    path: &Path,
    context: &TsFactContext,
    source: &str,
    needs_semantic: bool,
) -> (bool, bool) {
    let record_waits = waits_in_scope(
        path,
        &context.root,
        &context.queue_factory_roles.wait_scopes,
    );
    let build_semantic = needs_semantic
        && (needs_job_id_scoping(source)
            || needs_enqueue_scoping(source, &context.queue_factory_roles)
            || needs_rate_limit_scoping(source)
            || needs_scheduler_scoping(source, &context.queue_factory_roles)
            || (record_waits && needs_wait_scoping(source, &context.queue_factory_roles)));
    (record_waits, build_semantic)
}

use super::{TsFactContext, TsFactPlan};
use crate::queue::extract::{
    needs_enqueue_scoping, needs_job_id_scoping, needs_rate_limit_scoping, needs_scheduler_scoping,
    needs_wait_scoping, waits_in_scope,
};
use oxc_ast::ast::Program;
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use std::path::Path;

pub(super) fn prepare<'a>(
    path: &Path,
    plan: TsFactPlan,
    context: &TsFactContext,
    source: &str,
    program: &'a Program<'a>,
    needs_semantic: bool,
) -> (bool, Option<SemanticBuilderReturn<'a>>) {
    let record_waits = plan.queue_project
        && waits_in_scope(
            path,
            &context.root,
            &context.queue_factory_roles.wait_scopes,
        );
    let semantic = (needs_semantic
        && plan.queue_project
        && (needs_job_id_scoping(source)
            || (context.queue_factory_roles.record_job_state
                && crate::queue::job_state::needs_scoping(source))
            || needs_enqueue_scoping(source, &context.queue_factory_roles)
            || needs_rate_limit_scoping(source)
            || needs_scheduler_scoping(source, &context.queue_factory_roles)
            || (record_waits && needs_wait_scoping(source, &context.queue_factory_roles))))
    .then(|| {
        SemanticBuilder::new()
            .with_build_nodes(true)
            .with_check_syntax_error(true)
            .build(program)
    });
    (record_waits, semantic)
}

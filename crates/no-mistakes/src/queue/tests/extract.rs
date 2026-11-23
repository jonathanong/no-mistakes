use crate::queue::extract::{
    extract_program_with_scoping, needs_job_id_scoping, needs_scheduler_scoping,
    needs_wait_scoping, waits_in_scope, FileFacts,
};
use oxc_ast::ast::Program;
use std::path::Path;

pub(crate) fn extract_program_with_factories(
    path: &Path,
    source: &str,
    program: &Program<'_>,
    factory_names: &[String],
    roles: &crate::config::v2::schema::QueueFactoryRoles,
    root: &Path,
) -> FileFacts {
    let record_waits = waits_in_scope(path, root, &roles.wait_scopes);
    let semantic = (needs_job_id_scoping(source)
        || needs_scheduler_scoping(source, roles)
        || (record_waits && needs_wait_scoping(source, roles)))
    .then(|| oxc_semantic::SemanticBuilder::new().build(program));
    extract_program_with_scoping(
        path,
        source,
        program,
        factory_names,
        roles,
        record_waits,
        semantic.as_ref().map(|built| built.semantic.scoping()),
    )
}

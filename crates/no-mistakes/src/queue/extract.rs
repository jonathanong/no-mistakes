use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use oxc_semantic::Scoping;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub(crate) use crate::queue::extract_model::{FileFacts, ProducerSite, WorkerSite};

pub(crate) fn needs_wait_scoping(
    source: &str,
    roles: &crate::config::v2::schema::QueueFactoryRoles,
) -> bool {
    !roles.enqueue_and_wait.is_empty()
        || !roles.enqueue_and_wait_patterns.is_empty()
        || source.contains("const")
            && (source.contains("addAndWait")
                || source.contains("waitUntilFinished")
                || source.contains('\\'))
}

pub(crate) use super::enqueue_wrapper::needs_scoping as needs_enqueue_scoping;

pub(crate) fn needs_job_id_scoping(source: &str) -> bool {
    source.contains("jobId") || source.contains('\\')
}

pub(crate) fn needs_rate_limit_scoping(source: &str) -> bool {
    let bytes = source.as_bytes();
    super::extract_prefilter::contains_bytes(bytes, b"import")
        && (super::extract_prefilter::contains_bytes(bytes, b"rateLimit")
            || super::extract_prefilter::contains_bytes(bytes, b"RateLimitError")
            || bytes.contains(&b'\\'))
}

pub(crate) fn needs_scheduler_scoping(
    source: &str,
    roles: &crate::config::v2::schema::QueueFactoryRoles,
) -> bool {
    !roles.job_scheduler.is_empty()
        && (source.contains('\\')
            || roles
                .job_scheduler
                .iter()
                .any(|role| source.contains(&role.callee)))
}

pub(crate) fn extract_program_with_scoping(
    path: &Path,
    source: &str,
    program: &Program<'_>,
    factory_names: &[String],
    roles: &crate::config::v2::schema::QueueFactoryRoles,
    record_waits: bool,
    scoping: Option<&Scoping>,
) -> FileFacts {
    extract_queue_facts(
        path,
        source,
        program,
        factory_names,
        roles,
        record_waits,
        scoping,
    )
    .facts
}

fn extract_queue_facts<'a>(
    path: &'a Path,
    source: &'a str,
    program: &'a Program<'a>,
    factory_names: &'a [String],
    roles: &'a crate::config::v2::schema::QueueFactoryRoles,
    record_waits: bool,
    scoping: Option<&'a Scoping>,
) -> QueueVisitor<'a> {
    let scoped_scheduler = scoping.is_some() && needs_scheduler_scoping(source, roles);
    let index = super::extract_factory::queue_index(program, roles);
    let scheduler_factories = if scoped_scheduler {
        super::extract_factory::scheduler_factories(program, roles)
    } else {
        Default::default()
    };
    let wrapper_mutations = super::enqueue_wrapper::Mutations::collect(
        program,
        &index.object_bindings,
        roles.enqueue.iter().any(|role| role.positional().is_none()),
    );
    let mut visitor = QueueVisitor {
        path,
        source,
        facts: FileFacts::default(),
        const_strings: HashMap::new(),
        job_id_consts: Default::default(),
        job_id_flows: Default::default(),
        job_id_queues: Default::default(),
        job_id_pending: Vec::new(),
        binding_is_const: false,
        queue_classes: HashSet::new(),
        worker_classes: HashSet::new(),
        flow_classes: HashSet::new(),
        flow_bindings: HashSet::new(),
        non_queue_declarations: HashSet::new(),
        configured_factory_names: factory_names,
        roles,
        queue_name_index: HashMap::new(),
        wrapper_mutations,
        wrapper_factories: super::enqueue_wrapper::factories(program, roles, scoping),
        enqueue_factories: HashMap::new(),
        worker_factories: HashMap::new(),
        scheduler_factories,
        object_bindings: index.object_bindings,
        job_frames: Vec::new(),
        record_waits,
        scoping,
        wait_aliases: Default::default(),
        claimed_processors: Vec::new(),
        current_case: None,
        namespace_imports: HashMap::new(),
        processor_spans: index.processor_spans,
        dispatch_dynamic_workers: HashSet::new(),
        dispatch_jobs: HashMap::new(),
        dispatch_scopes: HashSet::new(),
        function_stack: Vec::new(),
        top_level_callables: 0,
        worker_stack: Vec::new(),
        worker_rate_limit: Default::default(),
        shared_rate_limit: super::extract_visitor::RateLimitState::new(
            scoping.is_some() && needs_rate_limit_scoping(source),
        ),
    };
    super::number_scope::seed_program(&mut visitor, program);
    visitor.visit_program(program);
    if !visitor.non_queue_declarations.is_empty() {
        visitor.facts.producers.retain(|producer| {
            !matches!(
                producer.binding,
                crate::queue::extract_model::ProducerBinding::Local(at)
                    if visitor.non_queue_declarations.contains(&at)
            )
        });
    }
    super::enqueue_wrapper::collect_exports(&mut visitor, program);
    super::extract_job_id::finish(&mut visitor);
    if roles.record_job_state && super::job_state::needs_scoping(source) {
        super::job_state::finish(&mut visitor, program);
    }
    super::extract_worker_rate_limit::finish(&mut visitor, program);
    visitor.finish_shared_rate_limits();
    super::number_scope::finish_opaque(&mut visitor.facts);
    super::extract_processor::finish(&mut visitor);
    super::job_options::finish(&mut visitor.facts, &visitor.object_bindings);
    visitor
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;

#[path = "wait_scope.rs"]
mod wait_scope;
pub(crate) use wait_scope::{waits_in_scope, WaitScopes};

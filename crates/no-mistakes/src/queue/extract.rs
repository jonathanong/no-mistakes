use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use oxc_semantic::Scoping;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub(crate) use crate::queue::extract_model::{FileFacts, ProducerSite, WorkerSite};

pub(crate) fn needs_wait_scoping(source: &str) -> bool {
    source.contains("const")
        && (source.contains("addAndWait")
            || source.contains("waitUntilFinished")
            || source.contains('\\'))
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
    let mut visitor = QueueVisitor {
        path,
        source,
        facts: FileFacts::default(),
        const_strings: HashMap::new(),
        queue_classes: HashSet::new(),
        worker_classes: HashSet::new(),
        flow_classes: HashSet::new(),
        flow_bindings: HashSet::new(),
        configured_factory_names: factory_names,
        roles,
        queue_name_index: HashMap::new(),
        enqueue_factories: HashMap::new(),
        worker_factories: HashMap::new(),
        job_frames: Vec::new(),
        record_waits,
        scoping,
        wait_aliases: Default::default(),
        claimed_processors: Vec::new(),
        current_case: None,
        namespace_imports: HashMap::new(),
        processor_spans: super::extract_factory::processor_spans(program, roles),
        dispatch_dynamic_workers: HashSet::new(),
        dispatch_jobs: HashMap::new(),
        dispatch_scopes: HashSet::new(),
        function_stack: Vec::new(),
        top_level_callables: 0,
        worker_stack: Vec::new(),
    };
    visitor.visit_program(program);
    super::extract_processor::finish(&mut visitor);
    visitor.facts
}

#[path = "wait_scope.rs"]
mod wait_scope;
pub(crate) use wait_scope::{waits_in_scope, WaitScopes};

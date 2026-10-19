use crate::queue::extract_helpers::collect_jobs_from_text;
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub(crate) use crate::queue::extract_model::{FileFacts, ProducerSite, WorkerSite};

pub(crate) fn extract_program_with_factories(
    path: &Path,
    source: &str,
    program: &Program<'_>,
    factory_names: &[String],
    roles: &crate::config::v2::schema::QueueFactoryRoles,
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
        in_job: 0,
        current_case: None,
        namespace_imports: HashMap::new(),
        processor_spans: Vec::new(),
        worker_stack: Vec::new(),
    };
    visitor.visit_program(program);
    let mut jobs = collect_jobs_from_text(source);
    jobs.sort();
    jobs.dedup();
    visitor.facts.referenced_jobs = jobs;
    visitor.facts
}

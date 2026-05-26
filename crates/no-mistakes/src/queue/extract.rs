use crate::ast;
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub(crate) use crate::queue::extract_model::{FileFacts, ProducerSite, WorkerSite};

pub(crate) fn extract_file_with_factories(
    path: &Path,
    factory_names: &[String],
) -> anyhow::Result<FileFacts> {
    let source = std::fs::read_to_string(path)?;
    ast::with_program(path, &source, |program, _| {
        extract_program_with_factories(path, &source, program, factory_names)
    })
}

pub(crate) fn extract_program(path: &Path, source: &str, program: &Program<'_>) -> FileFacts {
    extract_program_with_factories(path, source, program, &[])
}

pub(crate) fn extract_program_with_factories(
    path: &Path,
    source: &str,
    program: &Program<'_>,
    factory_names: &[String],
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
        factory_functions: HashSet::new(),
        configured_factory_names: factory_names,
        namespace_imports: HashMap::new(),
    };
    visitor.visit_program(program);
    visitor.facts
}

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
        job_frames: Vec::new(),
        record_waits: waits_in_scope(path, &roles.wait_scopes),
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

pub(crate) fn waits_in_scope(
    path: &Path,
    projects: &[crate::config::v2::schema::ProjectWaitScopes],
) -> bool {
    let Some(project) = containing_project(path, projects) else {
        return true;
    };
    scopes_match(path, &project.scopes)
}

fn containing_project<'a>(
    path: &Path,
    projects: &'a [crate::config::v2::schema::ProjectWaitScopes],
) -> Option<&'a crate::config::v2::schema::ProjectWaitScopes> {
    projects
        .iter()
        .filter(|project| root_contains(path, &project.root))
        .max_by_key(|project| project.root.len())
}

fn root_contains(path: &Path, root: &str) -> bool {
    let root = root.trim().trim_matches('/');
    if root.is_empty() || root == "." {
        return true;
    }
    let path = path.to_string_lossy().replace('\\', "/");
    path == root
        || path.starts_with(&format!("{root}/"))
        || path.contains(&format!("/{root}/"))
        || path.ends_with(&format!("/{root}"))
}

fn scopes_match(path: &Path, scopes: &[String]) -> bool {
    if scopes.is_empty() {
        return true;
    }
    let mut builder = globset::GlobSetBuilder::new();
    let mut added = false;
    for scope in scopes {
        for pattern in [scope.clone(), format!("**/{scope}")] {
            let Ok(glob) = globset::GlobBuilder::new(&pattern)
                .literal_separator(false)
                .build()
            else {
                continue;
            };
            builder.add(glob);
            added = true;
        }
    }
    if !added {
        return true;
    }
    builder.build().is_ok_and(|set| set.is_match(path))
}

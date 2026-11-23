use super::super::extract_model::{
    FileFacts, JobOptionEvidence, JobOptionSite, LocalQueue, QueueDefaultOptions,
};
use super::super::extract_visitor::QueueVisitor;
use super::super::source::line_number;
use super::site::{evidence_at, push_site};
use crate::queue::object_bindings::ObjectBindings;
use oxc_ast::ast::{Argument, CallExpression, Expression};
use oxc_span::GetSpan;
use std::collections::HashMap;
use std::path::PathBuf;

#[path = "job_options_default_read.rs"]
mod read;

pub(crate) fn note_factory_defaults(
    visitor: &mut QueueVisitor<'_>,
    declaration: u32,
    queue_name: Option<&str>,
    name_index: usize,
    call: &CallExpression<'_>,
) {
    let line = line_number(visitor.source, call.span().start);
    let mut recorded = blank(declaration, line, queue_name.map(str::to_string));
    for (index, arg) in call.arguments.iter().enumerate() {
        if index == name_index {
            continue;
        }
        note_options_argument(arg, &visitor.object_bindings, &mut recorded);
    }
    visitor.facts.queue_defaults.push(recorded);
}

pub(crate) fn note_constructed_defaults(
    visitor: &mut QueueVisitor<'_>,
    declaration: u32,
    new_expr: &oxc_ast::ast::NewExpression<'_>,
) {
    let line = line_number(visitor.source, new_expr.span().start);
    let mut recorded = blank(
        declaration,
        line,
        new_expr
            .arguments
            .first()
            .and_then(|arg| visitor.literal_arg(arg)),
    );
    if let Some(arg) = new_expr.arguments.get(1) {
        note_options_argument(arg, &visitor.object_bindings, &mut recorded);
    }
    visitor.facts.queue_defaults.push(recorded);
}

pub(crate) fn note_argument_options(
    visitor: &mut QueueVisitor<'_>,
    binding: &str,
    job: &Option<String>,
    call: &CallExpression<'_>,
    index: Option<usize>,
) {
    push_site(
        visitor,
        call.span().start,
        Some(binding.to_string()),
        None,
        job.clone(),
        index.map_or_else(JobOptionEvidence::default, |index| {
            evidence_at(call, index, &visitor.object_bindings)
        }),
    );
}

fn blank(declaration: u32, line: usize, queue_name: Option<String>) -> QueueDefaultOptions {
    QueueDefaultOptions {
        declaration,
        line,
        queue_name,
        evidence: JobOptionEvidence::default(),
    }
}

fn note_options_argument(
    arg: &Argument<'_>,
    bindings: &ObjectBindings<'_>,
    recorded: &mut QueueDefaultOptions,
) {
    let Some(expr) = arg.as_expression() else {
        recorded.evidence = JobOptionEvidence::open();
        return;
    };
    if !matches!(expr.get_inner_expression(), Expression::StringLiteral(_)) {
        recorded.evidence = read::queue_defaults(expr, bindings);
    }
}

enum IndexedDefaults {
    Single(JobOptionEvidence),
    Ambiguous,
}

pub(super) struct DefaultsIndex {
    by_file_queue: HashMap<(PathBuf, String), IndexedDefaults>,
    by_declaration: HashMap<(PathBuf, u32), JobOptionEvidence>,
}

impl DefaultsIndex {
    pub(super) fn lookup_evidence(
        &self,
        site: &JobOptionSite,
    ) -> Option<Result<&JobOptionEvidence, ()>> {
        let file = site.queue_file.as_ref()?;
        if let Some(declaration) = site.declaration {
            if let Some(evidence) = self.by_declaration.get(&(file.clone(), declaration)) {
                return Some(Ok(evidence));
            }
        }
        let name = site.queue_name.as_ref()?;
        match self.by_file_queue.get(&(file.clone(), name.clone()))? {
            IndexedDefaults::Single(evidence) => Some(Ok(evidence)),
            IndexedDefaults::Ambiguous => Some(Err(())),
        }
    }
}

pub(super) fn index_defaults(facts: &HashMap<PathBuf, FileFacts>) -> DefaultsIndex {
    let mut by_file_queue: HashMap<(PathBuf, String), IndexedDefaults> = HashMap::new();
    let mut by_declaration: HashMap<(PathBuf, u32), JobOptionEvidence> = HashMap::new();
    for (path, file) in facts {
        for defaults in &file.queue_defaults {
            let name = defaults.queue_name.clone().or_else(|| {
                let LocalQueue::Known(name) = file.queue_declarations.get(&defaults.declaration)?
                else {
                    return None;
                };
                Some(name.clone())
            });
            let Some(name) = name else {
                continue;
            };
            by_declaration.insert(
                (path.clone(), defaults.declaration),
                defaults.evidence.clone(),
            );
            by_file_queue
                .entry((path.clone(), name))
                .and_modify(|existing| *existing = IndexedDefaults::Ambiguous)
                .or_insert_with(|| IndexedDefaults::Single(defaults.evidence.clone()));
        }
    }
    DefaultsIndex {
        by_file_queue,
        by_declaration,
    }
}

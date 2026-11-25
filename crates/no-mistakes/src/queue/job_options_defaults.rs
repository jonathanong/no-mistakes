use super::super::extract_model::{
    FileFacts, JobOptionEvidence, JobOptionSite, LocalQueue, ProducerBinding, QueueDefaultOptions,
};
use super::super::extract_visitor::QueueVisitor;
use super::super::source::line_number;
use super::site::{evidence_at, push_site};
use oxc_ast::ast::{Argument, CallExpression, Expression};
use oxc_span::GetSpan;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[path = "job_options_default_read.rs"]
mod read;

pub(crate) fn note_factory_defaults<'a>(
    visitor: &mut QueueVisitor<'a>,
    declaration: u32,
    queue_name: Option<&str>,
    name_index: usize,
    call: &CallExpression<'a>,
) {
    let line = line_number(visitor.source, call.span().start);
    let mut recorded = blank(declaration, line, queue_name.map(str::to_string));
    for (index, arg) in call.arguments.iter().enumerate() {
        if index == name_index {
            continue;
        }
        note_options_argument(visitor, arg, line, &mut recorded);
    }
    visitor.facts.queue_defaults.push(recorded);
}

pub(crate) fn note_constructed_defaults<'a>(
    visitor: &mut QueueVisitor<'a>,
    declaration: u32,
    new_expr: &oxc_ast::ast::NewExpression<'a>,
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
        note_options_argument(visitor, arg, line, &mut recorded);
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

fn note_options_argument<'a>(
    visitor: &mut QueueVisitor<'a>,
    arg: &Argument<'a>,
    line: usize,
    recorded: &mut QueueDefaultOptions,
) {
    let Some(expr) = arg.as_expression() else {
        recorded.evidence = JobOptionEvidence::open();
        return;
    };
    if !matches!(expr.get_inner_expression(), Expression::StringLiteral(_)) {
        recorded.evidence = read::queue_defaults(expr, &visitor.object_bindings);
        crate::queue::extract_debounce::check_queue_options(visitor, expr, line);
    }
}

enum IndexedDefaults {
    Single(JobOptionEvidence),
    Ambiguous,
}

pub(super) struct DefaultsIndex {
    by_file_queue: HashMap<(PathBuf, String), IndexedDefaults>,
    by_declaration: HashMap<(PathBuf, u32), JobOptionEvidence>,
    unresolved_declarations: HashMap<PathBuf, HashSet<u32>>,
}

impl DefaultsIndex {
    pub(super) fn has_unresolved_queue(&self, path: &Path, site: &JobOptionSite) -> bool {
        matches!(site.binding, Some(ProducerBinding::Local(declaration))
            if self
                .unresolved_declarations
                .get(path)
                .is_some_and(|declarations| declarations.contains(&declaration)))
    }

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

pub(super) fn resolved_default_queue_name(
    file: &FileFacts,
    defaults: &super::super::extract_model::QueueDefaultOptions,
) -> Option<String> {
    defaults.queue_name.clone().or_else(|| {
        let LocalQueue::Known(name) = file.queue_declarations.get(&defaults.declaration)? else {
            return None;
        };
        Some(name.clone())
    })
}

pub(super) fn index_defaults(facts: &HashMap<PathBuf, FileFacts>) -> DefaultsIndex {
    let mut by_file_queue: HashMap<(PathBuf, String), IndexedDefaults> = HashMap::new();
    let mut by_declaration: HashMap<(PathBuf, u32), JobOptionEvidence> = HashMap::new();
    let mut unresolved_declarations: HashMap<PathBuf, HashSet<u32>> = HashMap::new();
    for (path, file) in facts {
        for defaults in &file.queue_defaults {
            let Some(name) = resolved_default_queue_name(file, defaults) else {
                unresolved_declarations
                    .entry(path.clone())
                    .or_default()
                    .insert(defaults.declaration);
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
        unresolved_declarations,
    }
}

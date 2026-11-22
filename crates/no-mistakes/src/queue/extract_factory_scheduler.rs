use crate::queue::extract_job_id;
use crate::queue::extract_model::LocalCheck;
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::lazy_object_bindings::LazyObjectBindings;
use crate::queue::source::line_number;
use oxc_ast::ast::{CallExpression, Expression, ObjectExpression, ObjectPropertyKind};

const KIND: &str = "scheduler-template-option";

pub(super) fn note_import(visitor: &mut QueueVisitor<'_>, imported: &str, local: &str) {
    if let Some(spec) = visitor
        .roles
        .job_scheduler
        .iter()
        .find(|spec| spec.callee == imported)
    {
        visitor
            .scheduler_factories
            .insert(local.to_string(), spec.template);
    }
}

pub(super) fn note_scheduler(visitor: &mut QueueVisitor<'_>, call: &CallExpression<'_>) {
    let Some(index) = template_index(visitor, call) else {
        return;
    };
    let Some(argument) = call
        .arguments
        .get(index)
        .and_then(|argument| argument.as_expression())
    else {
        return;
    };
    let Some(found) = resolve(argument, &visitor.object_bindings, call.span.start, 0) else {
        return;
    };
    let Some(hits) = collect_template(found, visitor.source, &visitor.object_bindings) else {
        return;
    };
    for (name, line) in hits {
        visitor.facts.checks.push(LocalCheck {
            line,
            kind: KIND.to_string(),
            job: None,
            message: format!(
                "`{name}` is set on a job-scheduler template. The scheduler's repeat settings control timing and identity."
            ),
        });
    }
}

fn template_index(visitor: &QueueVisitor<'_>, call: &CallExpression<'_>) -> Option<usize> {
    let callee = crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee);
    match callee {
        Expression::StaticMemberExpression(member)
            if member.property.name == "upsertJobScheduler" =>
        {
            Some(2)
        }
        Expression::ComputedMemberExpression(member)
            if crate::ast::computed_property_name(member).as_deref()
                == Some("upsertJobScheduler") =>
        {
            Some(2)
        }
        Expression::Identifier(id) => visitor
            .scheduler_factories
            .get(id.name.as_str())
            .copied()
            .filter(|_| extract_job_id::is_imported_callee(visitor, callee)),
        _ => None,
    }
}

struct Found<'a> {
    object: &'a ObjectExpression<'a>,
    at: u32,
}

fn resolve<'expr, 'arena: 'expr>(
    expr: &'expr Expression<'expr>,
    bindings: &'expr LazyObjectBindings<'arena>,
    at: u32,
    depth: u8,
) -> Option<Found<'expr>> {
    if depth > 8 {
        return None;
    }
    let expr = crate::codebase::ts_source::unwrap_ts_wrappers(expr);
    match expr {
        Expression::ObjectExpression(object) => Some(Found { object, at }),
        Expression::Identifier(id) => {
            let binding = bindings.lookup(id.name.as_str(), at)?;
            resolve(binding.value?, bindings, binding.resolve_at, depth + 1)
        }
        _ => None,
    }
}

type LocatedOptions<'a> = (&'a Expression<'a>, u32);

fn collect_template<'expr, 'arena: 'expr>(
    found: Found<'expr>,
    source: &str,
    bindings: &'expr LazyObjectBindings<'arena>,
) -> Option<Vec<(String, usize)>> {
    let mut hits = Vec::new();
    let mut opts = None;
    collect(found, source, bindings, &mut hits, Some(&mut opts))?;
    if let Some((value, at)) = opts {
        let options = resolve(value, bindings, at, 0)?;
        collect(options, source, bindings, &mut hits, None)?;
    }
    Some(hits)
}

fn collect<'expr, 'arena: 'expr>(
    found: Found<'expr>,
    source: &str,
    bindings: &'expr LazyObjectBindings<'arena>,
    hits: &mut Vec<(String, usize)>,
    mut opts: Option<&mut Option<LocatedOptions<'expr>>>,
) -> Option<()> {
    for property in &found.object.properties {
        match property {
            ObjectPropertyKind::SpreadProperty(spread) => {
                let spread = resolve(&spread.argument, bindings, found.at, 0)?;
                collect(spread, source, bindings, hits, opts.as_deref_mut())?;
            }
            ObjectPropertyKind::ObjectProperty(property) => {
                let key = crate::ast::property_key_name(&property.key)?;
                if key == "opts" {
                    if let Some(opts) = opts.as_deref_mut() {
                        *opts = Some((&property.value, found.at));
                    }
                } else if forbidden(key.as_ref()) {
                    hits.push((key.into_owned(), line_number(source, property.span.start)));
                }
            }
        }
    }
    Some(())
}

fn forbidden(name: &str) -> bool {
    matches!(name, "delay" | "deduplication" | "parent" | "jobId")
}

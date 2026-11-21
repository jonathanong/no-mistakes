use crate::queue::extract_model::LocalCheck;
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_ast::ast::{CallExpression, Expression, ObjectExpression, ObjectPropertyKind};

#[path = "extract_factory_scheduler_bind.rs"]
mod bind;

use bind::lookup;
pub use bind::{collect_object_bindings, ObjectBinding};

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
    let Some(hits) = collect(
        found.object,
        visitor.source,
        &visitor.object_bindings,
        found.at,
        true,
    ) else {
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
        Expression::Identifier(id) => visitor.scheduler_factories.get(id.name.as_str()).copied(),
        _ => None,
    }
}

struct Found<'a> {
    object: &'a ObjectExpression<'a>,
    at: u32,
}

fn resolve<'a>(
    expr: &'a Expression<'a>,
    bindings: &'a [ObjectBinding<'a>],
    at: u32,
    depth: u8,
) -> Option<Found<'a>> {
    if depth > 8 {
        return None;
    }
    let expr = crate::codebase::ts_source::unwrap_ts_wrappers(expr);
    match expr {
        Expression::ObjectExpression(object) => Some(Found { object, at }),
        Expression::Identifier(id) => {
            let binding = lookup(bindings, id.name.as_str(), at)?;
            resolve(binding.value?, bindings, binding.resolve_at, depth + 1)
        }
        _ => None,
    }
}

fn collect(
    object: &ObjectExpression<'_>,
    source: &str,
    bindings: &[ObjectBinding<'_>],
    at: u32,
    template: bool,
) -> Option<Vec<(String, usize)>> {
    let mut hits = Vec::new();
    for property in &object.properties {
        match property {
            ObjectPropertyKind::SpreadProperty(spread) => {
                let found = resolve(&spread.argument, bindings, at, 0)?;
                hits.extend(collect(found.object, source, bindings, found.at, template)?);
            }
            ObjectPropertyKind::ObjectProperty(property) => {
                let key = crate::ast::property_key_name(&property.key)?;
                if template && key == "opts" {
                    let found = resolve(&property.value, bindings, at, 0)?;
                    hits.extend(collect(found.object, source, bindings, found.at, false)?);
                } else if !(template && key == "data") && forbidden(key.as_ref()) {
                    hits.push((key.into_owned(), line_number(source, property.span.start)));
                }
            }
        }
    }
    Some(hits)
}

fn forbidden(name: &str) -> bool {
    matches!(name, "delay" | "deduplication" | "parent" | "jobId")
}

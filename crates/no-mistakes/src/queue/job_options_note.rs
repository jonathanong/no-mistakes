use super::super::extract_helpers::binding_name;
use super::super::extract_model::{JobOptionEvidence, ProducerBinding};
use super::super::extract_visitor::QueueVisitor;
use super::site::{evidence_at, job_at, member_string, push_site};
use super::walk::{read_member, read_object};
use oxc_ast::ast::{
    ArrayExpressionElement, CallExpression, Expression, ObjectExpression, VariableDeclaration,
};
use oxc_span::GetSpan;

pub(crate) fn note_consts(visitor: &mut QueueVisitor<'_>, decl: &VariableDeclaration<'_>) {
    for declarator in &decl.declarations {
        let Some(name) = binding_name(&declarator.id) else {
            continue;
        };
        let Some(init) = &declarator.init else {
            continue;
        };
        match init.get_inner_expression() {
            Expression::ObjectExpression(object) => record_const(&mut visitor.facts, name, object),
            Expression::Identifier(id) => {
                visitor.facts.const_objects.insert(
                    name,
                    JobOptionEvidence {
                        spreads: vec![id.name.to_string()],
                        ..JobOptionEvidence::default()
                    },
                );
            }
            _ => {}
        }
    }
}

fn record_const(
    facts: &mut super::super::extract_model::FileFacts,
    name: String,
    object: &ObjectExpression<'_>,
) {
    if let Some(opts) = read_member(object, "opts") {
        facts.const_opts.insert(name.clone(), opts);
    }
    facts.const_objects.insert(name, read_object(object));
}

pub(crate) fn note_call<'a>(visitor: &mut QueueVisitor<'a>, call: &CallExpression<'a>) {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return;
    };
    let binding = match &member.object {
        Expression::Identifier(id) => Some(id.name.to_string()),
        other => visitor.imported_member_binding(other),
    };
    match member.property.name.as_str() {
        "add"
            if binding.as_deref().is_some_and(|name| {
                matches!(
                    visitor.producer_binding(name, call.span.start),
                    ProducerBinding::Local(at) if visitor.flow_bindings.contains(&at)
                )
            }) =>
        {
            super::flow::note_flow(visitor, call);
        }
        "add" => {
            if binding.is_some() {
                note_add(visitor, call, binding);
            }
        }
        "addBulk" => note_bulk(visitor, call, binding),
        "upsertJobScheduler" => super::flow::note_scheduler(visitor, call, binding),
        _ => {}
    }
}

fn note_add(visitor: &mut QueueVisitor<'_>, call: &CallExpression<'_>, binding: Option<String>) {
    push_site(
        visitor,
        call.span().start,
        binding,
        None,
        job_at(visitor, call, 0),
        evidence_at(call, 2),
    );
}

fn note_bulk(visitor: &mut QueueVisitor<'_>, call: &CallExpression<'_>, binding: Option<String>) {
    let Some(Expression::ArrayExpression(array)) = call
        .arguments
        .first()
        .and_then(oxc_ast::ast::Argument::as_expression)
        .map(Expression::get_inner_expression)
    else {
        push_site(
            visitor,
            call.span().start,
            binding,
            None,
            None,
            JobOptionEvidence::open(),
        );
        return;
    };
    for element in &array.elements {
        match element {
            ArrayExpressionElement::ObjectExpression(object) => {
                note_bulk_item(visitor, object, binding.clone());
            }
            ArrayExpressionElement::SpreadElement(spread) => push_site(
                visitor,
                spread.span().start,
                binding.clone(),
                None,
                None,
                JobOptionEvidence::open(),
            ),
            _ => push_site(
                visitor,
                element.span().start,
                binding.clone(),
                None,
                None,
                JobOptionEvidence::open(),
            ),
        }
    }
}

fn note_bulk_item(
    visitor: &mut QueueVisitor<'_>,
    object: &ObjectExpression<'_>,
    binding: Option<String>,
) {
    push_site(
        visitor,
        object.span().start,
        binding,
        None,
        member_string(object, "name"),
        read_member(object, "opts").unwrap_or_default(),
    );
}

use crate::queue::extract_model::LocalCheck;
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_ast::ast::{
    Argument, BindingIdentifier, BindingPattern, CallExpression, Expression, FormalParameters,
    ObjectPropertyKind, VariableDeclaration, VariableDeclarationKind,
};

#[path = "extract_factory_dedup_ids.rs"]
mod ids;
use oxc_ast_visit::Visit;

pub(crate) fn note_self_dedup(
    visitor: &mut QueueVisitor<'_>,
    call: &CallExpression<'_>,
    recorded: usize,
) {
    let job = call
        .arguments
        .first()
        .and_then(|arg| visitor.literal_arg(arg));
    let Some(expr) = call.arguments.get(2).and_then(Argument::as_expression) else {
        return;
    };
    note_dedup_argument(visitor, call, expr, job, recorded);
}

pub(super) fn note_dedup_argument(
    visitor: &mut QueueVisitor<'_>,
    call: &CallExpression<'_>,
    expr: &Expression<'_>,
    job: Option<String>,
    recorded: usize,
) {
    if debounce_with_ttl(expr) {
        visitor.facts.checks.push(LocalCheck {
            line: line_number(visitor.source, call.span.start),
            kind: "debounce-ttl".to_string(),
            job: job.clone(),
            message: "deduplication ttl is ignored when mode is debounce".to_string(),
        });
    }
    if visitor.job_frames.is_empty() {
        return;
    }
    let Some(job) = job else {
        return;
    };
    if visitor
        .current_case
        .as_ref()
        .is_some_and(|case| case != &job)
    {
        return;
    }
    if !dedup_uses_active_job(visitor, expr) {
        return;
    }
    for producer in &mut visitor.facts.producers[recorded..] {
        producer.self_dedup = true;
    }
}

fn debounce_with_ttl(expr: &Expression<'_>) -> bool {
    let Expression::ObjectExpression(object) = expr else {
        return false;
    };
    object.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        crate::ast::property_key_name(&property.key).as_deref() == Some("deduplication")
            && debounce_object(&property.value)
    })
}

fn debounce_object(expr: &Expression<'_>) -> bool {
    let Expression::ObjectExpression(object) = expr else {
        return false;
    };
    let mut debounce = false;
    let mut ttl = false;
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        match crate::ast::property_key_name(&property.key).as_deref() {
            Some("mode") => {
                debounce = matches!(
                    &property.value,
                    Expression::StringLiteral(value) if value.value == "debounce"
                );
            }
            Some("ttl") => ttl = true,
            _ => {}
        }
    }
    debounce && ttl
}

fn dedup_uses_active_job(visitor: &QueueVisitor<'_>, expr: &Expression<'_>) -> bool {
    let Expression::ObjectExpression(object) = expr else {
        return false;
    };
    object.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        match crate::ast::property_key_name(&property.key).as_deref() {
            Some("jobId") => visitor.tracks_job_token(&property.value),
            Some("deduplication") => dedup_id_token(visitor, &property.value),
            _ => false,
        }
    })
}

fn dedup_id_token(visitor: &QueueVisitor<'_>, expr: &Expression<'_>) -> bool {
    let Expression::ObjectExpression(object) = expr else {
        return visitor.tracks_job_token(expr);
    };
    object.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        crate::ast::property_key_name(&property.key).as_deref() == Some("id")
            && visitor.tracks_job_token(&property.value)
    })
}

pub(crate) fn visit_variable_declaration<'a>(
    visitor: &mut QueueVisitor<'a>,
    decl: &VariableDeclaration<'a>,
) {
    for declarator in &decl.declarations {
        let token = decl.kind == VariableDeclarationKind::Const
            && declarator
                .init
                .as_ref()
                .is_some_and(|expr| visitor.tracks_job_token(expr));
        mask_pattern(visitor, &declarator.id);
        if token {
            if let (Some(name), Some(frame)) = (
                crate::queue::extract_helpers::binding_name(&declarator.id),
                visitor.job_frames.last_mut(),
            ) {
                frame.id_idents.push(name);
            }
        }
        if let Some(init) = declarator.init.as_ref() {
            if decl.kind == VariableDeclarationKind::Const {
                ids::note_destructured_ids(visitor, &declarator.id, init);
            }
        }
        visitor.visit_variable_declarator(declarator);
    }
}

pub(super) fn mask_parameters(visitor: &mut QueueVisitor<'_>, params: &FormalParameters<'_>) {
    for param in &params.items {
        mask_pattern(visitor, &param.pattern);
    }
    if let Some(rest) = &params.rest {
        mask_pattern(visitor, &rest.rest.argument);
    }
}

pub(super) fn mask_pattern(visitor: &mut QueueVisitor<'_>, pattern: &BindingPattern<'_>) {
    struct Names(Vec<String>);
    impl<'a> Visit<'a> for Names {
        fn visit_binding_identifier(&mut self, id: &BindingIdentifier<'a>) {
            self.0.push(id.name.to_string());
        }
    }
    let mut names = Names(Vec::new());
    names.visit_binding_pattern(pattern);
    for frame in &mut visitor.job_frames {
        if frame
            .object
            .as_ref()
            .is_some_and(|name| names.0.contains(name))
        {
            frame.object = None;
        }
        frame.id_idents.retain(|name| !names.0.contains(name));
        frame.name_idents.retain(|name| !names.0.contains(name));
    }
}

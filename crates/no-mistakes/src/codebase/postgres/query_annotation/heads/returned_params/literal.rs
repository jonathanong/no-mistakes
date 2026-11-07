//! Literal-only append chains returning an exact first-parameter alias.
use super::{span, Candidate};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    BindingPattern, Expression, Function, FunctionType, Statement, VariableDeclarationKind,
};

pub(super) fn function(value: &Function<'_>) -> Option<Candidate> {
    if value.r#type != FunctionType::FunctionDeclaration || value.r#async || value.generator {
        return None;
    }
    let params = &value.params;
    let [first] = params.items.as_slice() else {
        return None;
    };
    if params.rest.is_some() || !first.decorators.is_empty() || first.initializer.is_some() {
        return None;
    }
    let BindingPattern::BindingIdentifier(parameter) = &first.pattern else {
        return None;
    };
    let body = value.body.as_ref()?;
    let [Statement::VariableDeclaration(declaration), Statement::ReturnStatement(ret)] =
        body.statements.as_slice()
    else {
        return None;
    };
    if declaration.kind != VariableDeclarationKind::Const || declaration.declarations.len() != 1 {
        return None;
    }
    let binding = &declaration.declarations[0];
    let BindingPattern::BindingIdentifier(alias) = &binding.id else {
        return None;
    };
    let Expression::Identifier(returned) = unwrap_ts_wrappers(ret.argument.as_ref()?) else {
        return None;
    };
    if returned.name != alias.name {
        return None;
    }
    let mut expression = binding.init.as_ref()?;
    let mut tags = Vec::new();
    let mut appends = 0;
    while let Expression::CallExpression(call) = unwrap_ts_wrappers(expression) {
        if appends == 8 {
            return None;
        }
        let receiver = super::super::append::literal_receiver(call)?;
        match unwrap_ts_wrappers(call.arguments.first()?.as_expression()?) {
            Expression::StringLiteral(_) => {}
            Expression::TaggedTemplateExpression(template)
                if template.quasi.expressions.is_empty() && template.quasi.quasis.len() == 1 =>
            {
                let Expression::Identifier(tag) = unwrap_ts_wrappers(&template.tag) else {
                    return None;
                };
                tags.push(span(tag.span));
            }
            _ => return None,
        }
        expression = receiver;
        appends += 1;
    }
    let Expression::Identifier(receiver) = unwrap_ts_wrappers(expression) else {
        return None;
    };
    if appends == 0 || receiver.name != parameter.name {
        return None;
    }
    Some(Candidate {
        parameter: span(parameter.span),
        alias: span(alias.span),
        allowed_parameter: span(receiver.span),
        allowed_alias: vec![span(returned.span)],
        tags,
        function: span(value.span),
        body: span(body.span),
    })
}

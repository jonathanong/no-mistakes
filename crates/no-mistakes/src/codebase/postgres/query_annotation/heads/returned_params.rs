//! Source-proven identity of a returned native builder parameter.
use super::Span;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    BindingPattern, Expression, Function, FunctionType, Statement, VariableDeclarationKind,
};

mod caller;
mod projection;
pub(super) use caller::root;
pub(super) use projection::{module_projection, Pending};

#[derive(Clone, Copy)]
pub(super) struct Root {
    pub callee: Span,
    pub tag: Span,
    pub proof: super::HeadProof,
}

#[derive(Clone)]
pub(super) struct Candidate {
    pub parameter: Span,
    pub alias: Span,
    pub allowed_parameter: Span,
    pub allowed_alias: Vec<Span>,
    pub default_tag: Span,
    pub function: Span,
    pub body: Span,
}

pub(super) fn function(value: &Function<'_>) -> Option<Candidate> {
    if value.r#type != FunctionType::FunctionDeclaration || value.r#async || value.generator {
        return None;
    }
    let params = &value.params;
    if params.rest.is_some()
        || params.items.len() != 3
        || params.items.iter().any(|p| !p.decorators.is_empty())
    {
        return None;
    }
    let [first, extra_parameter, from_parameter] = params.items.as_slice() else {
        return None;
    };
    let (
        BindingPattern::BindingIdentifier(parameter),
        BindingPattern::BindingIdentifier(extra),
        BindingPattern::BindingIdentifier(from),
    ) = (
        &first.pattern,
        &extra_parameter.pattern,
        &from_parameter.pattern,
    )
    else {
        return None;
    };
    if first.initializer.is_some() || extra_parameter.initializer.is_some() {
        return None;
    }
    let Expression::TaggedTemplateExpression(default) =
        unwrap_ts_wrappers(from_parameter.initializer.as_ref()?)
    else {
        return None;
    };
    if !default.quasi.expressions.is_empty() || default.quasi.quasis.len() != 1 {
        return None;
    }
    let Expression::Identifier(tag) = unwrap_ts_wrappers(&default.tag) else {
        return None;
    };
    let body = value.body.as_ref()?;
    // This complete body shape makes the extra guards false for a one-arg call
    // and leaves the final alias return as its only same-function return.
    let [Statement::VariableDeclaration(declaration), first_if, from_append, second_if, Statement::ReturnStatement(ret)] =
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
    let parameter_use = append_chain(binding.init.as_ref()?)?;
    if parameter_use.0 != parameter.name.as_str() {
        return None;
    }
    let first_alias = guarded_append(first_if, extra.name.as_str(), alias.name.as_str())?;
    let from_alias = discarded_append(from_append, alias.name.as_str(), Some(from.name.as_str()))?;
    let second_alias = guarded_append(second_if, extra.name.as_str(), alias.name.as_str())?;
    let Expression::Identifier(returned) = unwrap_ts_wrappers(ret.argument.as_ref()?) else {
        return None;
    };
    if returned.name != alias.name {
        return None;
    }
    Some(Candidate {
        parameter: span(parameter.span),
        alias: span(alias.span),
        allowed_parameter: parameter_use.1,
        allowed_alias: vec![first_alias, from_alias, second_alias, span(returned.span)],
        default_tag: span(tag.span),
        function: span(value.span),
        body: span(body.span),
    })
}

fn append_chain<'a>(mut expression: &'a Expression<'a>) -> Option<(&'a str, Span)> {
    let mut appends = 0;
    while let Expression::CallExpression(call) = unwrap_ts_wrappers(expression) {
        if appends == 8 {
            return None;
        }
        expression = super::append::prefix_receiver(call)?;
        appends += 1;
    }
    let Expression::Identifier(id) = unwrap_ts_wrappers(expression) else {
        return None;
    };
    (appends > 0).then_some((id.name.as_str(), span(id.span)))
}

fn guarded_append(statement: &Statement<'_>, guard: &str, alias: &str) -> Option<Span> {
    let Statement::IfStatement(branch) = statement else {
        return None;
    };
    if branch.alternate.is_some() {
        return None;
    }
    let Expression::Identifier(condition) = unwrap_ts_wrappers(&branch.test) else {
        return None;
    };
    (condition.name == guard)
        .then(|| discarded_append(&branch.consequent, alias, None))
        .flatten()
}

fn discarded_append(statement: &Statement<'_>, alias: &str, arg: Option<&str>) -> Option<Span> {
    let Statement::ExpressionStatement(statement) = statement else {
        return None;
    };
    let Expression::CallExpression(call) = unwrap_ts_wrappers(&statement.expression) else {
        return None;
    };
    let receiver = super::append::prefix_receiver(call)?;
    let Expression::Identifier(id) = unwrap_ts_wrappers(receiver) else {
        return None;
    };
    if id.name != alias {
        return None;
    }
    if let Some(expected) = arg {
        let Expression::Identifier(actual) =
            unwrap_ts_wrappers(call.arguments.first()?.as_expression()?)
        else {
            return None;
        };
        if actual.name != expected {
            return None;
        }
    }
    Some(span(id.span))
}

fn span(value: oxc_span::Span) -> Span {
    (value.start, value.end)
}

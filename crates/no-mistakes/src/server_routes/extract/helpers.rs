use crate::server_routes::model::Binding;
use crate::server_routes::types::Framework;
use oxc_ast::ast::{Argument, BindingPattern, Expression, ObjectPropertyKind};

impl Binding {
    pub(super) fn new(framework: Framework, prefix: Option<String>) -> Self {
        Self {
            framework,
            prefixes: prefix.into_iter().collect(),
        }
    }
}

pub(super) fn binding_name(pattern: &BindingPattern<'_>) -> Option<String> {
    if let BindingPattern::BindingIdentifier(id) = pattern {
        return Some(id.name.as_str().to_string());
    }
    None
}

pub(super) fn binding_names(pattern: &BindingPattern<'_>) -> Vec<String> {
    match pattern {
        BindingPattern::BindingIdentifier(id) => vec![id.name.to_string()],
        BindingPattern::ObjectPattern(object) => {
            let mut names = Vec::new();
            for prop in &object.properties {
                names.extend(binding_names(&prop.value));
            }
            if let Some(rest) = &object.rest {
                names.extend(binding_names(&rest.argument));
            }
            names
        }
        BindingPattern::ArrayPattern(array) => {
            let mut names = Vec::new();
            for element in array.elements.iter().flatten() {
                names.extend(binding_names(element));
            }
            if let Some(rest) = &array.rest {
                names.extend(binding_names(&rest.argument));
            }
            names
        }
        BindingPattern::AssignmentPattern(assign) => binding_names(&assign.left),
    }
}

pub(super) fn first_object_prefix(args: &[Argument<'_>]) -> Option<String> {
    let Argument::ObjectExpression(object) = args.first()? else {
        return None;
    };
    for property in &object.properties {
        if let ObjectPropertyKind::ObjectProperty(property) = property {
            if let (Some("prefix"), Expression::StringLiteral(value)) =
                (property.key.static_name().as_deref(), &property.value)
            {
                return Some(value.value.as_str().to_string());
            }
        }
    }
    None
}

pub(super) fn object_identifier(object: &Expression<'_>) -> Option<String> {
    match object {
        Expression::Identifier(id) => Some(id.name.to_string()),
        Expression::ParenthesizedExpression(expr) => object_identifier(&expr.expression),
        _ => None,
    }
}

pub(super) fn mounted_binding(arg: &Argument<'_>) -> Option<String> {
    let expr = arg.as_expression()?;
    if let Expression::CallExpression(call) = expr {
        match &call.callee {
            Expression::StaticMemberExpression(member)
                if matches!(member.property.name.as_str(), "routes" | "middleware") =>
            {
                return object_identifier(&member.object);
            }
            _ => {}
        }
    }
    object_identifier(expr)
}

pub(super) fn method_name(method: &str) -> String {
    if method == "del" {
        "delete".to_string()
    } else {
        method.to_ascii_lowercase()
    }
}

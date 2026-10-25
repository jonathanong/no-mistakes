//! Declaration-only constructor identity references, extracted in the fact pass.
use oxc_ast::ast::*;

pub(super) fn aliases(declaration: &VariableDeclarator<'_>) -> Vec<(String, String)> {
    let BindingPattern::BindingIdentifier(name) = &declaration.id else {
        return Vec::new();
    };
    let Some(annotation) = &declaration.type_annotation else {
        return Vec::new();
    };
    let prefix = name.name.as_str();
    match &annotation.type_annotation {
        TSType::TSTypeQuery(query) => query_name(&query.expr_name)
            .map(|target| vec![(prefix.into(), target)])
            .unwrap_or_default(),
        TSType::TSTypeLiteral(literal) => literal
            .members
            .iter()
            .filter_map(|member| {
                let TSSignature::TSPropertySignature(property) = member else {
                    return None;
                };
                if property.computed || property.optional {
                    return None;
                }
                let member = crate::codebase::ts_source::static_property_key_name(&property.key)?;
                let TSType::TSTypeQuery(query) =
                    &property.type_annotation.as_ref()?.type_annotation
                else {
                    return None;
                };
                Some((format!("{prefix}.{member}"), query_name(&query.expr_name)?))
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn query_name(name: &TSTypeQueryExprName<'_>) -> Option<String> {
    match name {
        TSTypeQueryExprName::IdentifierReference(name) => Some(name.name.to_string()),
        TSTypeQueryExprName::QualifiedName(name) => {
            Some(format!("{}.{}", type_name(&name.left)?, name.right.name))
        }
        _ => None,
    }
}
fn type_name(name: &TSTypeName<'_>) -> Option<String> {
    match name {
        TSTypeName::IdentifierReference(name) => Some(name.name.to_string()),
        TSTypeName::QualifiedName(name) => {
            Some(format!("{}.{}", type_name(&name.left)?, name.right.name))
        }
        _ => None,
    }
}

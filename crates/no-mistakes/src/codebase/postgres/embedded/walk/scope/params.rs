use oxc_ast::ast::{TSType, TSTypeName};

pub(super) fn param_is_sql_statement(
    param: &oxc_ast::ast::FormalParameter<'_>,
    bindings: &std::collections::HashSet<String>,
) -> bool {
    let Some(annotation) = &param.type_annotation else {
        return false;
    };
    let TSType::TSTypeReference(reference) = &annotation.type_annotation else {
        return false;
    };
    matches!(
        &reference.type_name,
        TSTypeName::IdentifierReference(identifier) if bindings.contains(identifier.name.as_str())
    )
}

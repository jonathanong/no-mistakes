fn last_object_property<'ast, 'source>(
    object: &'ast jsonc_parser::ast::Object<'source>,
    name: &str,
) -> Option<&'ast jsonc_parser::ast::Object<'source>> {
    object
        .properties
        .iter()
        .rev()
        .find(|property| property.name.as_str() == name)
        .and_then(|property| property.value.as_object())
}

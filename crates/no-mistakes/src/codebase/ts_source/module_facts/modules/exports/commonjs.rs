use super::*;

pub(in crate::codebase::ts_source::module_facts::modules) fn commonjs_reference(
    value: &IdentifierReference<'_>,
    node_id: oxc_semantic::NodeId,
    semantic: &Semantic<'_>,
    facts: &mut TypeScriptModuleFacts,
) {
    if value.name != "module" && value.name != "exports" {
        return;
    }
    let reference = semantic.scoping().get_reference(
        value
            .reference_id
            .get()
            .expect("semantic CommonJS reference"),
    );
    if !reference.is_value()
        || reference.flags().is_value_as_type()
        || loads::has_runtime_binding(reference, semantic)
    {
        return;
    }
    if value.name == "module" {
        let property = match loads::unwrapped_parent(node_id, semantic) {
            Some(AstKind::StaticMemberExpression(member)) => {
                Some(std::borrow::Cow::Borrowed(member.property.name.as_str()))
            }
            Some(AstKind::ComputedMemberExpression(member)) => {
                crate::ast::computed_property_name(member)
            }
            _ => None,
        };
        if property.is_some_and(|name| name != "exports") {
            return;
        }
    }
    unsupported(
        facts,
        value.span,
        "CommonJS module/export objects are unsupported",
    );
}

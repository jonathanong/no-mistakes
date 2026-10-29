use super::*;

const UNSUPPORTED: &str = "CommonJS module/export objects are unsupported";

/// A top-level `module.exports = ...` or `exports.name = ...` statement.
pub(in crate::codebase::ts_source::module_facts::modules) struct ExportAssignment {
    module: bool,
    span: oxc_span::Span,
}

pub(in crate::codebase::ts_source::module_facts::modules) fn commonjs_reference(
    value: &IdentifierReference<'_>,
    node_id: oxc_semantic::NodeId,
    semantic: &Semantic<'_>,
    assignments: &mut Vec<ExportAssignment>,
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
    if top_level_assignment(value, node_id, semantic) {
        assignments.push(ExportAssignment {
            module: value.name == "module",
            span: value.span,
        });
    }
    unsupported(facts, value.span, UNSUPPORTED);
}

/// One lone `module.exports = value`, or only `exports.name = value`
/// statements, are the file's exports and cannot hide an importer. Any other
/// combination is unsupported for importer analysis too.
pub(in crate::codebase::ts_source::module_facts::modules) fn finish_commonjs(
    assignments: Vec<ExportAssignment>,
    facts: &mut TypeScriptModuleFacts,
) {
    let modules = assignments.iter().filter(|a| a.module).count();
    let accepted = modules == 0 || assignments.len() == 1;
    if accepted {
        for assignment in assignments {
            facts
                .importer_safe
                .push((assignment.span.start, assignment.span.end));
        }
    }
}

fn top_level_assignment(
    value: &IdentifierReference<'_>,
    node_id: oxc_semantic::NodeId,
    semantic: &Semantic<'_>,
) -> bool {
    let nodes = semantic.nodes();
    let AstKind::StaticMemberExpression(member) = nodes.parent_kind(node_id) else {
        return false;
    };
    if member.optional || (value.name == "module" && member.property.name != "exports") {
        return false;
    }
    let member_id = nodes.parent_id(node_id);
    let AstKind::AssignmentExpression(assignment) = nodes.parent_kind(member_id) else {
        return false;
    };
    let assignment_id = nodes.parent_id(member_id);
    assignment.operator == AssignmentOperator::Assign
        && assignment.left.span() == member.span
        && matches!(
            nodes.parent_kind(assignment_id),
            AstKind::ExpressionStatement(_)
        )
        && matches!(
            nodes.parent_kind(nodes.parent_id(assignment_id)),
            AstKind::Program(_)
        )
}

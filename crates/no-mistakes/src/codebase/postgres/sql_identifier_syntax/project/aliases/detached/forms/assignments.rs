use super::*;
use oxc_ast::ast::{AssignmentExpression, AssignmentTarget};

pub(super) fn collect(collector: &mut Collector, value: &AssignmentExpression<'_>) {
    if !value.operator.is_assign() {
        return;
    }
    let AssignmentTarget::AssignmentTargetIdentifier(target) = &value.left else {
        return;
    };
    let Some(id) = collector.owners.get(&span(target.span)).copied() else {
        return;
    };
    let Some((source, form, _, uses)) = collector.form(&value.right) else {
        return;
    };
    collector.nodes.push(Node {
        id,
        source,
        form,
        exact: false,
        uses,
    });
    collector.node_ids.insert(id);
}

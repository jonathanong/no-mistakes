use super::*;
use oxc_ast::ast::{AssignmentExpression, AssignmentTarget};

pub(super) fn collect(collector: &mut Collector<'_>, value: &AssignmentExpression<'_>) {
    if !value.operator.is_assign() {
        return;
    }
    let AssignmentTarget::AssignmentTargetIdentifier(target) = &value.left else {
        return;
    };
    let Some(id) = collector.owners.get(&span(target.span)).copied() else {
        return;
    };
    for profile in 0..collector.profiles.len() {
        let Some((source, form, _, uses)) = collector.form(&value.right, profile) else {
            continue;
        };
        collector.nodes.push(Node {
            profile,
            id,
            source,
            form,
            exact: false,
            uses,
        });
        collector.node_ids.insert(id);
    }
}

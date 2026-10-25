use super::*;
use oxc_ast::ast::{AssignmentExpression, AssignmentTarget};
pub(super) fn record(
    tracker: &mut Tracker,
    value: &AssignmentExpression<'_>,
    ids: &FxHashMap<Span, usize>,
) -> Node {
    let source = span(value.right.span());
    match &value.left {
        AssignmentTarget::AssignmentTargetIdentifier(id) => {
            if let Some(id) = ids.get(&span(id.span)) {
                tracker.flows.push((*id, source, Vec::new()));
            } else {
                tracker.barriers.push(source);
            }
        }
        AssignmentTarget::StaticMemberExpression(member) => {
            store(tracker, &member.object, source, ids)
        }
        AssignmentTarget::ComputedMemberExpression(member) => {
            store(tracker, &member.object, source, ids)
        }
        _ => tracker.barriers.push(source),
    }
    Node::Copy(source)
}
fn store(
    tracker: &mut Tracker,
    object: &Expression<'_>,
    source: Span,
    ids: &FxHashMap<Span, usize>,
) {
    if let Some(id) = chain(object).and_then(|(root, _)| ids.get(&root)) {
        tracker.stores.push((*id, source));
    } else {
        tracker.barriers.push(source);
    }
}

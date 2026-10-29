use super::*;

pub(super) fn member(
    collector: &mut Collector<'_>,
    at: Span,
    receiver: &Expression<'_>,
    name: String,
) {
    if !collector
        .profiles
        .iter()
        .any(|profile| profile.executor_names.contains(&name))
    {
        return;
    }
    let Some((root, parts)) = chain(receiver) else {
        return;
    };
    if !parts.is_empty() && collector.owners.contains_key(&root) {
        collector.members.push(Member {
            span: at,
            root,
            name,
        });
    }
}

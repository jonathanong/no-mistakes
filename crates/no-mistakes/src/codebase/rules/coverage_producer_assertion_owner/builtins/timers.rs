use super::*;

pub(super) fn classify<'a>(
    site: &ResolvedCallSite,
    node_sites: &[&'a ResolvedCallSite],
    calls: &CallIndex<'_>,
) -> Callbacks<'a> {
    let mut out = Callbacks::default();
    let matched = calls
        .at_lexical_call(site)
        .iter()
        .copied()
        .filter(|call| call.source_callee == site.source_callee)
        .collect::<Vec<_>>();
    if matched.is_empty() {
        out.unresolved.push("timer callback".to_string());
    }
    for call in matched {
        collect_arguments(Some(call), &[0], node_sites, &mut out);
    }
    out
}

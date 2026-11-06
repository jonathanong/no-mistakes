const ENTRY_POINT_SCOPE: &str = "<entry-point>";

fn project_import_execution(inputs: &GraphEdgeBuildInputs<'_>, facts: &dyn TsFactLookup, pass: &mut CallPass) {
    let mut removed: FxHashMap<Edge, usize> = fx_map();
    let mut projected = Vec::new();
    for site in &mut pass.sites {
        if site.caller_id.is_some() { continue; }
        let Some(file) = facts.get_ts_facts(&site.file) else { continue; };
        let ranges = &file.import_inactive_call_ranges;
        let after = ranges.partition_point(|(start, _)| *start <= site.offset);
        if after == 0 || site.offset >= ranges[after - 1].1 { continue; }
        let previous = NodeId::file_in(&inputs.interner, &site.file);
        set_entry_point_owner(site, CallableId(ranges[after - 1].0));
        let source = NodeId::scoped_in(&inputs.interner, &site.file, ENTRY_POINT_SCOPE, site.caller_id);
        for target in site.target_nodes() {
            *removed.entry((previous.clone(), target.clone(), EdgeKind::Call)).or_default() += 1;
            projected.push((source.clone(), target.clone(), EdgeKind::Call));
        }
    }
    if !removed.is_empty() {
        pass.edges.retain(|edge| {
            if let Some(count) = removed.get_mut(edge) {
                if *count > 0 {
                    *count -= 1;
                    return false;
                }
            }
            true
        });
    }
    pass.edges.extend(projected);
}

fn entry_point_nodes<'a>(sites: &'a [ResolvedCallSite], interner: &'a PathInterner) -> impl Iterator<Item = NodeId> + 'a {
    sites.iter().filter(|site| site.caller.as_deref() == Some(ENTRY_POINT_SCOPE))
        .map(|site| NodeId::scoped_in(interner, &site.file, ENTRY_POINT_SCOPE, site.caller_id))
}

fn set_entry_point_owner(site: &mut ResolvedCallSite, owner: CallableId) {
    site.caller = Some(ENTRY_POINT_SCOPE.to_string());
    site.caller_id = Some(owner);
    for alternative in &mut site.alternatives {
        set_entry_point_owner(alternative, owner);
    }
}

use super::*;

type Owner = (std::path::PathBuf, CallableId);

pub(super) fn refine_callbacks(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    pass: &CallPass,
    callbacks: Vec<ResolvedCallSite>,
) -> Vec<ResolvedCallSite> {
    let mut incoming = refine::incoming_inputs_iter(
        pass.sites.iter().chain(callbacks.iter()),
        &pass.constructor_routes,
        indexes,
        facts,
    );
    let mut by_owner: FxHashMap<Owner, Vec<usize>> = fx_map();
    for (index, site) in pass.sites.iter().enumerate() {
        if let Some(id) = site.caller_id {
            by_owner
                .entry((site.file.clone(), id))
                .or_default()
                .push(index);
        }
    }
    let mut owner_pass = CallPass {
        sites: callbacks,
        constructor_routes: pass.constructor_routes.clone(),
        super_constructors: pass.super_constructors.clone(),
        replacement_files: pass.replacement_files.clone(),
        ..Default::default()
    };
    let callback_len = owner_pass.sites.len();
    let mut selected = Vec::new();
    refine::refine_value_calls_with_incoming(
        inputs,
        facts,
        resolver,
        indexes,
        &mut owner_pass,
        refine::WorklistInputs {
            incoming: &mut incoming,
            replacement_pass: Some(pass),
        },
        |sites| {
            let mut next = Vec::new();
            for site in sites {
                for target in site.target_nodes() {
                    if let NodeId::Symbol {
                        file,
                        callable_id: Some(id),
                        ..
                    } = target
                    {
                        if let Some(indices) = by_owner.remove(&(file.to_path_buf(), *id)) {
                            next.extend(indices);
                        }
                    }
                }
            }
            next.sort_unstable();
            let added = next
                .iter()
                .map(|index| pass.sites[*index].clone())
                .collect();
            selected.extend(next);
            added
        },
    );
    #[cfg(feature = "test-instrumentation")]
    super::super::super::value_flow_metrics::record_builtin_callback_sites(selected.len());
    let key = |site: &ResolvedCallSite| {
        (
            site.file.clone(),
            site.caller_id,
            site.offset,
            site.source_callee.clone(),
            site.invocation,
        )
    };
    let mut counts: FxHashMap<_, usize> = fx_map();
    for site in &pass.sites {
        *counts.entry(key(site)).or_default() += 1;
    }
    owner_pass
        .sites
        .into_iter()
        .enumerate()
        .filter_map(|(index, site)| {
            if counts.get(&key(&site)).is_some_and(|count| *count > 1) {
                return None;
            }
            (index < callback_len || {
                let original = &pass.sites[selected[index - callback_len]];
                original.target != site.target || original.target_node != site.target_node
            })
            .then_some(site)
        })
        .collect()
}

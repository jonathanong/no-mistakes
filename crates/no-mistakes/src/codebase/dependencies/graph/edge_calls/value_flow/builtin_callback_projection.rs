use super::*;

pub(in super::super) fn project_builtin_callbacks(
    inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    indexes: &CallableResolutionIndexes,
    pass: &mut CallPass,
) {
    let prepared: Vec<_> = inputs.graph_files.indexable().par_iter().filter_map(|path| {
        let file = facts.get_ts_facts(path)?;
        if !file.builtin_callback_calls.values().any(|proof| {
            proof.as_ref().is_some_and(|positions| !positions.is_empty())
        }) && !file.value_flow.calls.iter().any(|call| {
            matches!(call.source_callee.as_str(), "setTimeout" | "setInterval" | "setImmediate")
        }) {
            return None;
        }
        let base = indexes.file(facts, path)?;
        let mut owner = (*base).clone();
        owner.prepare_builtin_callbacks(file);
        (!owner.values.builtin_callbacks.is_empty() || !owner.values.native_callbacks.is_empty())
            .then(|| (path.clone(), std::sync::Arc::new(owner)))
    }).collect();
    if prepared.is_empty() { return; }
    let owner_indexes = CallableResolutionIndexes {
        files: std::sync::Arc::new(dashmap::DashMap::default()),
        namespace_roots: indexes.namespace_roots.clone(),
        external_namespaces: true,
        ..Default::default()
    };
    for entry in indexes.files.iter() {
        owner_indexes
            .files
            .insert(entry.key().clone(), entry.value().clone());
    }
    for (path, owner) in &prepared {
        owner_indexes.files.insert(path.clone(), owner.clone());
    }
    let resolved: Vec<_> = prepared.par_iter().map(|(path, owner)| {
        let resolution = CallSiteResolution {
            edge_inputs: inputs,
            facts,
            resolver,
            indexes: &owner_indexes,
            path,
            index: owner,
            namespaces_in_repo: false,
            escapes: Default::default(),
            opaque_external_members: Default::default(),
            call_prerequisites: Default::default(),
        };
        let sites = owner.values.builtin_callbacks.iter()
            .map(|call| resolution.resolve(call).1)
            .collect::<Vec<_>>();
        (sites, resolution.call_prerequisites.take(), resolution.opaque_external_members.take())
    }).collect();
    let mut sites = Vec::new();
    for ((path, _), (file_sites, prerequisites, opaque)) in prepared.iter().zip(resolved) {
        sites.extend(file_sites);
        for (offset, sources) in prerequisites {
            pass.call_prerequisites
                .entry(path.clone()).or_default()
                .entry(offset).or_default().extend(sources);
        }
        for (offset, members) in opaque {
            pass.opaque_external_members
                .entry(path.clone()).or_default()
                .entry(offset).or_default().extend(members);
        }
    }
    if sites.is_empty() { return; }
    let forwarded = prepared.iter().any(|(_, owner)| !owner.values.forwarded_callbacks.is_empty());
    if forwarded {
        let ordinary_len = pass.sites.len();
        let mut owner_pass = CallPass {
            sites: pass.sites.iter().cloned().chain(sites).collect(),
            constructor_routes: pass.constructor_routes.clone(),
            super_constructors: pass.super_constructors.clone(),
            replacement_files: pass.replacement_files.clone(),
            ..Default::default()
        };
        refine::refine_value_calls(inputs, facts, resolver, &owner_indexes, &mut owner_pass);
        let key = |site: &ResolvedCallSite| (site.file.clone(), site.caller_id, site.offset,
            site.source_callee.clone(), site.invocation);
        let mut counts: FxHashMap<_, usize> = fx_map();
        for site in &pass.sites { *counts.entry(key(site)).or_default() += 1; }
        sites = owner_pass.sites.into_iter().enumerate().filter_map(|(index, site)| {
            if counts.get(&key(&site)).is_some_and(|count| *count > 1) { return None; }
            (index >= ordinary_len || {
                let original = &pass.sites[index];
                original.target != site.target || original.target_node != site.target_node
            }).then_some(site)
        }).collect();
    } else {
        let incoming = refine::incoming_inputs(&pass.sites, &pass.constructor_routes, indexes, facts);
        let phases = execution_phases::ModuleWriterPhases::default();
        let cache = ParameterCache::default();
        sites.par_iter_mut().for_each(|site| {
            if site.target_node.is_some() { return; }
            refine::refine_site_in_place(
                inputs, facts, resolver, &owner_indexes, site,
                ValueInputs {
                    module_writer_phases: &phases,
                    cache: &cache,
                    incoming: &incoming,
                    supers: &pass.super_constructors,
                    replacements: None,
                },
                true,
            );
        });
    }
    for site in &sites {
        for target in site.target_nodes() {
            let source = site.caller.as_deref().map_or_else(
                || NodeId::file_in(&inputs.interner, &site.file),
                |caller| NodeId::scoped_in(&inputs.interner, &site.file, caller, site.caller_id),
            );
            pass.assertion_edges.push((
                source,
                target.clone(),
                EdgeKind::AssertionOwner,
            ));
        }
    }
    pass.assertion_sites = sites;
}

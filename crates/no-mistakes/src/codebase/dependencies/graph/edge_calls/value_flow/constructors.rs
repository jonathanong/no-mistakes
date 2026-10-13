use super::*;

pub(in super::super) fn collect_constructors(
    resolution: &CallSiteResolution<'_, '_>,
    file: &TsFileFacts,
    pass: &mut CallPass,
) {
    let ids = resolution
        .index
        .class_bindings
        .values()
        .flat_map(|names| names.values())
        .map(|class| class.class_id)
        .collect();
    pass.class_ids.push((resolution.path.to_path_buf(), ids));
    for call in file
        .function_calls
        .iter()
        .filter(|call| class_base_owner(call).is_some())
    {
        if let (Some((derived, base, _)), _) = resolution.resolve(call) {
            pass.constructor_bases.push((derived, base));
        }
    }
}

pub(in super::super) fn prepare_constructor_routes(pass: &mut CallPass) {
    let mut own: ConstructorRoutes = fx_map();
    for site in &pass.sites {
        if site.invocation == InvocationKind::Membership && site.source_callee == "constructor" {
            if let (Some(class), Some(target)) = (site.caller_id, &site.target_node) {
                own.entry(site.file.clone())
                    .or_default()
                    .insert(class, target.clone());
            }
        }
    }
    let classes: FxHashMap<_, _> = pass.class_ids.drain(..).collect();
    let mut bases: ConstructorRoutes = fx_map();
    for (derived, base) in &pass.constructor_bases {
        insert_route(&mut bases, derived, base.clone());
    }
    let mut routes = own.clone();
    for (derived, base) in pass.constructor_bases.drain(..) {
        let Some(target) = find_constructor(&base, &own, &bases, &classes) else {
            continue;
        };
        if let Some(constructor) = route(&own, &derived) {
            insert_route(&mut pass.super_constructors, constructor, target);
        } else {
            insert_route(&mut routes, &derived, target.clone());
            pass.edges.push((derived, target, EdgeKind::Call));
        }
    }
    pass.constructor_routes = routes;
}
fn find_constructor(
    node: &NodeId,
    own: &ConstructorRoutes,
    bases: &ConstructorRoutes,
    classes: &FxHashMap<std::path::PathBuf, FxHashSet<CallableId>>,
) -> Option<NodeId> {
    let mut current = node.clone();
    let mut visiting = fx_set();
    while visiting.insert(current.clone()) {
        if let Some(target) = route(own, &current) {
            return Some(target.clone());
        }
        if let Some(base) = route(bases, &current) {
            current = base.clone();
            continue;
        }
        let NodeId::Symbol {
            file,
            callable_id: Some(id),
            ..
        } = &current
        else {
            return None;
        };
        return (!classes
            .get(file.as_ref())
            .is_some_and(|classes| classes.contains(id)))
        .then_some(current);
    }
    None
}
fn insert_route(routes: &mut ConstructorRoutes, node: &NodeId, target: NodeId) {
    if let NodeId::Symbol {
        file,
        callable_id: Some(id),
        ..
    } = node
    {
        routes
            .entry(file.to_path_buf())
            .or_default()
            .insert(*id, target);
    }
}
pub(super) fn route<'a>(routes: &'a ConstructorRoutes, node: &NodeId) -> Option<&'a NodeId> {
    let NodeId::Symbol {
        file,
        callable_id: Some(id),
        ..
    } = node
    else {
        return None;
    };
    routes.get(file.as_ref())?.get(id)
}
impl ValueFlowResolution<'_, '_> {
    pub(super) fn super_constructor(&self, file: &std::path::Path, owner: CallableId) -> FlowValue {
        match self
            .supers
            .get(file)
            .and_then(|constructors| constructors.get(&owner))
        {
            Some(NodeId::Symbol {
                file,
                symbol,
                callable_id: Some(id),
            }) => FlowValue::Callable {
                file: file.to_path_buf(),
                scope: symbol.to_string(),
                id: *id,
            },
            _ => FlowValue::Unknown,
        }
    }
}

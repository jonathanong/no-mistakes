/// One resolution pass over every indexable file, split by what the plan
/// asked for: `Call` edges and call sites for `calls`, `Extends` edges and
/// class declarations for `extends`. A class base is never a call.
#[derive(Default)]
struct CallPass {
    edges: Vec<Edge>,
    sites: Vec<ResolvedCallSite>,
    assertion_edges: Vec<Edge>,
    assertion_sites: Vec<ResolvedCallSite>,
    opaque_external_members: FxHashMap<std::path::PathBuf, OpaqueExternalMembers>,
    call_prerequisites: FxHashMap<std::path::PathBuf, FailedCallSources>,
    replacement_files: Vec<(
        std::path::PathBuf,
        std::sync::Arc<crate::codebase::dependencies::extract::ValueReplacementFacts>,
    )>,
    classes: Vec<ClassDeclaration>,
    class_ids: Vec<(std::path::PathBuf, FxHashSet<CallableId>)>,
    constructor_bases: Vec<(NodeId, NodeId)>,
    constructor_routes: value_flow::ConstructorRoutes,
    super_constructors: value_flow::ConstructorRoutes,
    parameter_cache: value_flow::ParameterCache,
    modules: Vec<(String, std::path::PathBuf)>,
    /// Namespaces whose use the graph cannot follow to a class, per file.
    escapes: Vec<(std::path::PathBuf, String)>,
}

fn merge_call_passes(parts: Vec<CallPass>) -> CallPass {
    let mut all = CallPass {
        sites: Vec::with_capacity(parts.iter().map(|part| part.sites.len()).sum()),
        ..CallPass::default()
    };
    for part in parts {
        all.modules.extend(part.modules);
        all.edges.extend(part.edges);
        all.sites.extend(part.sites);
        all.opaque_external_members.extend(part.opaque_external_members);
        all.call_prerequisites.extend(part.call_prerequisites);
        all.replacement_files.extend(part.replacement_files);
        all.classes.extend(part.classes);
        all.class_ids.extend(part.class_ids);
        all.constructor_bases.extend(part.constructor_bases);
        all.escapes.extend(part.escapes);
    }
    all
}

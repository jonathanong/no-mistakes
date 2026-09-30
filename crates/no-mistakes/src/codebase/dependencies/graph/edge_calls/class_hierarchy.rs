/// A class's statically named `extends` base, resolved with the same rules as
/// a call. It is not a call site: nothing is invoked at the `extends` clause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedClassBase {
    pub file: std::path::PathBuf,
    /// Display scope of the derived class within `file`.
    pub class_scope: String,
    /// One-based line of the derived class declaration; zero when the source
    /// text was not available at extraction.
    pub line: u32,
    /// Whether the module exports the class under any name.
    pub exported: bool,
    pub base: ResolvedCallTarget,
}

/// Every class `extends` base and `new` site, for class-relationship rules.
/// Empty unless the graph was built with [`GraphBuildPlan::class_hierarchy`].
#[derive(Debug, Default)]
pub struct ClassHierarchy {
    pub bases: Vec<ResolvedClassBase>,
    pub constructions: Vec<ResolvedCallSite>,
}

impl DepGraph {
    pub fn class_hierarchy(&self) -> &ClassHierarchy {
        &self.class_hierarchy
    }
}

/// Resolves class bases and `new` sites, following `@scope/package` names into
/// the workspace as import edges already do. Only this pass does, so call
/// edges and call sites stay exactly as they are without it.
fn collect_class_hierarchy(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    workspace: &crate::codebase::workspaces::IndexedWorkspaceMap,
) -> ClassHierarchy {
    let resolver =
        crate::codebase::ts_resolver::WorkspaceFallbackResolver::new(resolver, workspace);
    // Export resolution depends on the resolver, so this pass keeps its own cache.
    let indexes = CallableResolutionIndexes::default();
    resolve_files(
        edge_inputs,
        facts,
        &resolver,
        &indexes,
        |resolution, file| {
            let unknown = resolution.unknown_sites(file);
            ClassHierarchy {
                bases: resolution.class_bases(file),
                constructions: file
                    .function_calls
                    .iter()
                    .filter(|call| {
                        call.invocation == InvocationKind::Construct
                            && is_traversable_call(resolution.index, call)
                    })
                    .map(|call| resolution.resolve(call).1)
                    .chain(unknown)
                    .filter(|site| site.invocation == InvocationKind::Construct)
                    .collect(),
            }
        },
    )
    .into_iter()
    .fold(ClassHierarchy::default(), |mut all, part| {
        all.bases.extend(part.bases);
        all.constructions.extend(part.constructions);
        all
    })
}

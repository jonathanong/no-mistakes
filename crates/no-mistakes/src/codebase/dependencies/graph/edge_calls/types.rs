/// A non-collapsing, statically resolved call occurrence. Graph edges dedupe
/// adjacency; policy consumers use these records when each source line matters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCallSite {
    pub file: std::path::PathBuf,
    pub caller: Option<String>,
    /// Opaque identity paired with `caller` for graph traversal only.
    pub caller_id: Option<crate::codebase::dependencies::extract::CallableId>,
    pub line: u32,
    /// Zero-based source byte where the invocation starts. This distinguishes
    /// multiple forbidden calls on the same source line.
    pub offset: u32,
    pub invocation: crate::codebase::dependencies::extract::InvocationKind,
    /// Exact source spelling, kept apart from canonical target selectors.
    pub source_callee: String,
    pub target: ResolvedCallTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedCallTarget {
    Global {
        name: String,
    },
    ModuleExport {
        specifier: String,
        export_path: String,
        repository_target: Option<(std::path::PathBuf, String)>,
        callable_id: Option<crate::codebase::dependencies::extract::CallableId>,
    },
    RepositoryFunction {
        file: std::path::PathBuf,
        scope: String,
    },
    Unknown,
}

fn module_export_target(
    file: &CallableFileIndex,
    callee: &str,
    repository_target: Option<(std::path::PathBuf, String)>,
    callable_id: Option<crate::codebase::dependencies::extract::CallableId>,
) -> Option<ResolvedCallTarget> {
    let (local, member) = callee
        .split_once('.')
        .map_or((callee, None), |(local, member)| (local, Some(member)));
    let binding = file.imported.get(local)?;
    let export_path = match binding.kind {
        crate::codebase::dependencies::extract::ImportedBindingKind::Namespace => {
            member?.to_string()
        }
        crate::codebase::dependencies::extract::ImportedBindingKind::Named => member
            .map(|member| format!("{}.{member}", binding.imported))
            .unwrap_or_else(|| binding.imported.clone()),
        crate::codebase::dependencies::extract::ImportedBindingKind::Default => member
            .map(|member| format!("default.{member}"))
            .unwrap_or_else(|| "default".to_string()),
    };
    Some(ResolvedCallTarget::ModuleExport {
        specifier: binding.specifier.clone(),
        export_path,
        repository_target,
        callable_id,
    })
}

fn graph_call_target_node(
    interner: &crate::codebase::analysis_session::PathInterner,
    facts: &dyn TsFactLookup,
    indexes: &CallableResolutionIndexes,
    target: &ResolvedCallTarget,
    local_callable_id: Option<crate::codebase::dependencies::extract::CallableId>,
) -> Option<NodeId> {
    match target {
        ResolvedCallTarget::RepositoryFunction { file, scope } => Some(callable_node_for_call(
            interner,
            facts,
            indexes,
            file,
            scope,
            local_callable_id,
        )),
        ResolvedCallTarget::ModuleExport {
            repository_target: Some((file, scope)),
            callable_id,
            ..
        } => Some(callable_node_for_call(
            interner,
            facts,
            indexes,
            file,
            scope,
            *callable_id,
        )),
        _ => None,
    }
}

fn callable_node_for_call(
    interner: &crate::codebase::analysis_session::PathInterner,
    facts: &dyn TsFactLookup,
    indexes: &CallableResolutionIndexes,
    file: &std::path::Path,
    scope: &str,
    exact_id: Option<crate::codebase::dependencies::extract::CallableId>,
) -> NodeId {
    let id = exact_id.or_else(|| {
        indexes
            .file(facts, file)
            .and_then(|index| index.unique_scope_id(scope))
    });
    id.map_or_else(
        || NodeId::symbol_in(interner, file, scope),
        |id| NodeId::callable_in(interner, file, scope, id),
    )
}

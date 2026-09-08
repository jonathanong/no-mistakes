/// A non-collapsing, statically resolved call occurrence. Graph edges dedupe
/// adjacency; policy consumers use these records when each source line matters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCallSite {
    pub file: std::path::PathBuf,
    pub caller: Option<String>,
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
    })
}

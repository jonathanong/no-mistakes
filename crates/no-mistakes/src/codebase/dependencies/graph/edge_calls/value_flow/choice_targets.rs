use super::*;

// Preserve #2071's branch metadata while sharing its callable nodes with the
// existing occurrence edge-replacement projection (not a second graph).
pub(super) fn target(sites: &[ResolvedCallSite]) -> ResolvedCallTarget {
    let mut known = Vec::new();
    let mut unknown = false;
    for site in sites {
        unknown |= site.has_unknown_target();
        for candidate in site.targets() {
            let target = match candidate.target {
                KnownCallTargetRef::Global { name } => {
                    KnownCallTarget::Global { name: name.into() }
                }
                KnownCallTargetRef::ModuleExport {
                    specifier,
                    export_path,
                    repository_target,
                    callable_id,
                } => KnownCallTarget::ModuleExport {
                    specifier: specifier.into(),
                    export_path: export_path.into(),
                    repository_target: repository_target
                        .map(|(file, scope)| (file.to_path_buf(), scope.into())),
                    callable_id,
                },
                KnownCallTargetRef::RepositoryFunction { file, scope } => {
                    KnownCallTarget::RepositoryFunction {
                        file: file.to_path_buf(),
                        scope: scope.into(),
                    }
                }
            };
            known.push(ResolvedCallCandidate {
                target,
                node: candidate.node.cloned(),
            });
        }
    }
    if known.is_empty() {
        return ResolvedCallTarget::Unknown;
    }
    ResolvedCallTarget::Alternatives(std::sync::Arc::new(ResolvedCallAlternatives::new(
        known, unknown,
    )))
}

#[cfg(test)]
#[path = "choice_targets_tests.rs"]
mod tests;

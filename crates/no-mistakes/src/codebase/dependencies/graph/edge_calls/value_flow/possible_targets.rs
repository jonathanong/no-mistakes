use super::*;

/// Retain authored branch metadata and every known candidate. Escaped uncertainty
/// adds no synthetic legacy branch, whose all-branches selector policy differs.
pub(super) fn widen(site: &mut ResolvedCallSite) {
    let known = site
        .targets()
        .map(|candidate| {
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
                    callable_id,
                    repository_target: repository_target
                        .map(|(path, name)| (path.to_path_buf(), name.into())),
                },
                KnownCallTargetRef::RepositoryFunction { file, scope } => {
                    KnownCallTarget::RepositoryFunction {
                        file: file.to_path_buf(),
                        scope: scope.into(),
                    }
                }
            };
            ResolvedCallCandidate {
                target,
                node: candidate.node.cloned(),
            }
        })
        .collect::<Vec<_>>();
    site.target = if known.is_empty() {
        ResolvedCallTarget::Unknown
    } else {
        ResolvedCallTarget::Alternatives(std::sync::Arc::new(ResolvedCallAlternatives::new(
            known, true,
        )))
    };
    site.target_node = None;
}

#[cfg(test)]
#[path = "possible_targets_tests.rs"]
mod tests;

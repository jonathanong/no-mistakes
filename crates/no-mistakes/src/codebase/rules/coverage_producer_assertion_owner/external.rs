//! External-module call classification for assertion-owner traversal.
use super::*;
use crate::codebase::dependencies::extract::InvocationKind;

/// Node `fs` specifiers. Calls through them are ordinary external I/O: they
/// are neither assertion evidence nor unresolved dispatch.
const FS_MODULES: [&str; 4] = ["fs", "node:fs", "fs/promises", "node:fs/promises"];

fn is_fs(candidate: &KnownCallTargetRef<'_>) -> bool {
    matches!(candidate, KnownCallTargetRef::ModuleExport { specifier, repository_target: None, .. }
        if FS_MODULES.contains(specifier))
}

/// Whether this occurrence calls a module outside the repository whose
/// behavior is not modeled. Pure `fs` I/O is modeled and returns false.
pub(super) fn unmodeled_external(site: &ResolvedCallSite) -> bool {
    site.targets().any(|candidate| {
        !is_fs(&candidate.target)
            && matches!(
                candidate.target,
                KnownCallTargetRef::ModuleExport {
                    repository_target: None,
                    ..
                }
            )
    })
}

pub(super) fn fs_effect(site: &ResolvedCallSite) -> bool {
    !site.has_unknown_target() && site.targets().any(|candidate| is_fs(&candidate.target))
}

/// `fs` may invoke a callback or promise continuation later; the canonical
/// graph has no resolved invocation for it, so such a node stays uncertain.
pub(super) fn hands_off_callback(sites: &[&ResolvedCallSite]) -> bool {
    sites.iter().any(|site| fs_effect(site))
        && sites
            .iter()
            .any(|site| site.invocation == InvocationKind::Callback)
}

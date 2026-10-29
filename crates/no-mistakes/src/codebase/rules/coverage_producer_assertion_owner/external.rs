//! External-module call classification for assertion-owner traversal.
use super::*;
use crate::codebase::dependencies::extract::InvocationKind;

/// Node `fs` specifiers. Calls through them are ordinary external I/O: they
/// are neither assertion evidence nor unresolved dispatch.
const FS_MODULES: [&str; 4] = ["fs", "node:fs", "fs/promises", "node:fs/promises"];

fn fs_export<'a>(candidate: &KnownCallTargetRef<'a>) -> Option<(&'a str, &'a str)> {
    match candidate {
        KnownCallTargetRef::ModuleExport {
            specifier,
            export_path,
            repository_target: None,
            ..
        } if FS_MODULES.contains(specifier) => Some((specifier, export_path)),
        _ => None,
    }
}

/// Whether this occurrence calls a module outside the repository whose
/// behavior is not modeled. Pure `fs` I/O is modeled and returns false.
pub(super) fn unmodeled_external(site: &ResolvedCallSite) -> bool {
    site.targets().any(|candidate| {
        fs_export(&candidate.target).is_none()
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
    !site.has_unknown_target() && site.targets().any(|c| fs_export(&c.target).is_some())
}

/// Whether a live `fs` occurrence may invoke a callback argument later. Only
/// callback-style APIs (neither `*Sync` nor promise-based) can; their last
/// argument being a non-literal value keeps the producer
/// uncertain because that invocation is not a resolved call.
pub(super) fn callback_argument(site: &ResolvedCallSite, facts: &dyn TsFactLookup) -> bool {
    let callback_style = site.targets().any(|candidate| {
        fs_export(&candidate.target).is_some_and(|(specifier, export)| {
            !specifier.ends_with("promises")
                && !export.ends_with("Sync")
                && !export.contains("promises.")
        })
    });
    callback_style
        && facts.get_ts_facts(&site.file).is_none_or(|facts| {
            facts
                .value_flow
                .calls
                .iter()
                .filter(|call| call.offset == site.offset)
                .any(|call| {
                    call.spread_at.is_some()
                        || (!call.arguments.is_empty() && !call.last_argument_literal)
                })
        })
}

/// Any synthetic callback site in a callable which also performs live `fs` I/O.
pub(super) fn has_callback_site(sites: &[&ResolvedCallSite]) -> bool {
    sites
        .iter()
        .any(|site| site.invocation == InvocationKind::Callback)
}

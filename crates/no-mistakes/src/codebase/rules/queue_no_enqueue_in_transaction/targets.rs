use super::analyze::Context;
use super::options::Api;
use super::reexports::{self, ResolvedApi};
use crate::codebase::dependencies::graph::{
    KnownCallTargetRef, ResolvedCallSite, ResolvedCallTarget,
};
use std::path::Path;

pub(super) type CallKey<'a> = (u32, &'a str);

pub(super) fn exact_target(site: &ResolvedCallSite) -> Option<Api<'_>> {
    if !site.alternatives.is_empty() {
        return None;
    }
    match &site.target {
        ResolvedCallTarget::ModuleExport {
            specifier,
            export_path,
            ..
        } => Some((specifier, export_path)),
        _ => None,
    }
}

pub(super) fn is_ambiguous(site: &ResolvedCallSite) -> bool {
    !site.alternatives.is_empty() || matches!(site.target, ResolvedCallTarget::Alternatives(_))
}

pub(super) fn any_candidate_matches(
    site: &ResolvedCallSite,
    predicate: impl Fn(Api<'_>) -> bool + Copy,
) -> bool {
    site.targets().any(|candidate| match candidate.target {
        KnownCallTargetRef::ModuleExport {
            specifier,
            export_path,
            ..
        } => predicate((specifier, export_path)),
        _ => false,
    }) || site
        .alternatives
        .iter()
        .any(|alt| any_candidate_matches(alt, predicate))
}

pub(super) fn site_key(site: &ResolvedCallSite) -> CallKey<'_> {
    (site.offset, site.source_callee.as_str())
}

pub(super) enum Boundary {
    Callback(usize),
    Begin,
    Unanalyzable,
    Other,
}

pub(super) fn resolve_boundary(ctx: &Context<'_>, file: &Path, api: Api<'_>) -> Boundary {
    match follow(ctx, file, api, |module, export| {
        ctx.targets.transactions.contains_key(&(module, export))
    }) {
        ResolvedApi::Target(module, export) => {
            Boundary::Callback(ctx.targets.transactions[&(module.as_str(), export.as_str())])
        }
        ResolvedApi::Unanalyzable => Boundary::Unanalyzable,
        ResolvedApi::Other => {
            match follow(ctx, file, api, |module, export| {
                ctx.targets.begins.contains(&(module, export))
            }) {
                ResolvedApi::Target(_, _) => Boundary::Begin,
                ResolvedApi::Unanalyzable => Boundary::Unanalyzable,
                ResolvedApi::Other => Boundary::Other,
            }
        }
    }
}

fn follow(
    ctx: &Context<'_>,
    file: &Path,
    api: Api<'_>,
    is_target: impl Fn(&str, &str) -> bool,
) -> ResolvedApi {
    reexports::follow_export(ctx, file, api.0, api.1, &is_target)
}

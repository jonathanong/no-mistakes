use super::analyze::{any_candidate_matches, is_ambiguous};
use super::analyze_callback::any_global_callback;
use super::receivers::FileEnv;
use super::reexports::{self, ResolvedApi};
use super::{scoped_finding, unanalyzable};
use crate::codebase::dependencies::graph::ResolvedCallSite;
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;

pub(super) fn ambiguous_publisher(env: &FileEnv<'_>, site: &ResolvedCallSite) -> bool {
    is_ambiguous(site)
        && (site.has_unknown_target()
            || any_global_callback(site)
            || any_candidate_matches(site, |api| {
                matches!(
                    reexports::follow_export(env.ctx, &site.file, api.0, api.1, &|module, export| {
                        env.ctx.targets.publishers.contains(&(module, export))
                    }),
                    ResolvedApi::Target(..) | ResolvedApi::Unanalyzable
                )
            }))
}

pub(super) fn report<'a>(
    env: &FileEnv<'_>,
    diagnostics: &mut AnalysisDiagnostics,
    candidates: impl Iterator<Item = (&'a ResolvedCallSite, Option<(String, String)>)>,
) {
    for (site, api) in candidates {
        if let Some((module, export)) = api {
            diagnostics.findings.push(scoped_finding(
                env.ctx,
                site,
                (module.as_str(), export.as_str()),
            ));
        } else {
            diagnostics.unanalyzable(env.ctx.policy, unanalyzable(env.ctx, site,
                "call target or execution timing is unresolved inside the transaction handle scope; call a statically known publisher after commit"));
        }
    }
}

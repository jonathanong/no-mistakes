use super::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::Path;
use std::sync::Arc;

mod plan;
mod playwright_source;
mod program;

use plan::{requires_parse, should_store_source, ts_source};
pub(crate) use program::collect_file_facts_from_program;

pub(crate) fn is_mdx_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("mdx")
}

pub(crate) fn collect_file_facts_with_sources(
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Option<CheckFileFacts> {
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    collect_file_facts_with_session_and_sources(&session, root, path, plan, playwright, sources)
}

pub(crate) fn collect_file_facts_with_session(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
) -> Option<CheckFileFacts> {
    let inventory = Arc::new(crate::codebase::ts_source::FileInventory::from_paths(&[
        path.to_path_buf(),
    ]));
    let sources = crate::codebase::ts_source::SourceStore::new(inventory);
    collect_file_facts_with_session_and_sources(session, root, path, plan, playwright, &sources)
}

pub(crate) fn collect_file_facts_with_session_and_sources(
    session: &crate::codebase::analysis_session::AnalysisSession,
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Option<CheckFileFacts> {
    let source = match sources.read_path(path) {
        Ok(source) => source,
        Err(err) => {
            let parse_error = format!("failed to read {}: {err}", path.display());
            return Some(CheckFileFacts {
                ts: Arc::new(TsFileFacts {
                    parse_error: Some(parse_error.clone()),
                    ..TsFileFacts::default()
                }),
                parse_error: Some(parse_error),
                server_route_client_boundary: plan
                    .server_route_client_boundary
                    .then(Default::default),
                ..CheckFileFacts::default()
            });
        }
    };
    if plan.storybook && is_mdx_file(path) {
        let stored_source = should_store_source(plan).then(|| Arc::clone(&source));
        return Some(CheckFileFacts {
            ts: Arc::new(ts_source(stored_source.clone())),
            source: stored_source,
            storybook: Some(crate::codebase::storybook::extract_mdx_source(&source)),
            // MDX is intentionally not sent through OXC, but every requested fact
            // family still needs an explicit entry for prepared consumers.
            server_route_client_boundary: plan.server_route_client_boundary.then(Default::default),
            ..CheckFileFacts::default()
        });
    }
    if plan.raw_source && !requires_parse(plan, path, playwright) {
        return Some(CheckFileFacts {
            ts: Arc::new(ts_source(Some(Arc::clone(&source)))),
            source: Some(Arc::clone(&source)),
            ..CheckFileFacts::default()
        });
    }
    if !requires_parse(plan, path, playwright) {
        return Some(CheckFileFacts::default());
    }
    let collected =
        session.with_recovered_program(path, &source, |program, parsed_source, parse_error| {
            if let Some(parse_error) = parse_error {
                let stored_source = should_store_source(plan).then(|| Arc::clone(&source));
                let ts = super::file_parse_error::ts_facts(
                    plan,
                    stored_source.clone(),
                    program,
                    parse_error.clone(),
                );
                let integration_runner_config =
                    plan.integration_runner_configs.as_ref().and_then(|plan| {
                        plan.parse_error(
                            path,
                            format!("failed to parse {}: {parse_error}", path.display()),
                        )
                    });
                return CheckFileFacts {
                    ts: Arc::new(ts),
                    source: stored_source,
                    integration_runner_config,
                    parse_error: Some(parse_error),
                    parsed: true,
                    server_route_client_boundary: plan
                        .server_route_client_boundary
                        .then(Default::default),
                    ..CheckFileFacts::default()
                };
            }
            let mut facts = collect_file_facts_from_program(
                root,
                path,
                plan,
                playwright,
                parsed_source,
                program,
            );
            if should_store_source(plan) {
                Arc::make_mut(&mut facts.ts).source = Some(source.to_string());
                facts.source = Some(Arc::clone(&source));
            }
            facts
        });
    match collected {
        Ok(facts) => Some(facts),
        Err(_) => {
            let stored_source = should_store_source(plan).then(|| Arc::clone(&source));
            let parse_error = format!("unsupported file type: {}", path.display());
            Some(CheckFileFacts {
                ts: Arc::new(TsFileFacts {
                    parse_error: Some(parse_error.clone()),
                    source: stored_source.as_deref().map(str::to_owned),
                    ..TsFileFacts::default()
                }),
                source: stored_source,
                parse_error: Some(parse_error),
                server_route_client_boundary: plan
                    .server_route_client_boundary
                    .then(Default::default),
                ..CheckFileFacts::default()
            })
        }
    }
}

use super::dynamic::callback_modules;
use super::fragments::{Catalog, Fragment};
use super::resolve::{binding_fragments, call_fragments, name_fragments, FileCtx};
use super::source_text::file_text;
use super::text::export_offset;
use super::RULE_ID;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use crate::codebase::postgres::embedded::TrustedSqlTag;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::{line_number, relative_slash_path};
use crate::fx::fx_map;
use std::path::Path;

pub(super) fn scan(
    root: &Path,
    tags: &[TrustedSqlTag],
    graph: &DepGraph,
    facts: &CheckFactMap,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) -> Vec<RuleFinding> {
    if !tags
        .iter()
        .any(|tag| !tag.module.is_empty() && !tag.name.is_empty())
    {
        return Vec::new();
    }
    let catalog = Catalog::build(facts, tags, sources);
    let mut findings = Vec::new();
    for file in facts.graph_file_universe() {
        let Some(ts) = facts.get_ts_facts(file) else {
            continue;
        };
        let Some(owned) = file_text(ts, sources, file) else {
            continue;
        };
        let source = owned.as_ref();
        let empty = fx_map();
        let probe = FileCtx {
            graph,
            facts,
            catalog: &catalog,
            tags,
            file,
            ts,
            source,
            callbacks: &empty,
            context: None,
        };
        let callbacks = callback_modules(&probe);
        let ctx = FileCtx {
            callbacks: &callbacks,
            ..probe
        };
        for (offset, fragments) in super::writes::assignment_sites(&ctx) {
            emit(&mut findings, root, &ctx, offset, &fragments);
        }
        for (offset, fragments) in super::namespace_uses::value_sites(&ctx) {
            emit(&mut findings, root, &ctx, offset, &fragments);
        }
        for call in &ts.value_flow.calls {
            emit(
                &mut findings,
                root,
                &ctx,
                call.offset,
                &call_fragments(&ctx, call),
            );
        }
        for binding in &ts.value_flow.bindings {
            if binding.owner.is_some() || !exported_local(ts, &binding.name) {
                continue;
            }
            emit(
                &mut findings,
                root,
                &ctx,
                binding.declared_at,
                &binding_fragments(&ctx, binding),
            );
        }
        for export in &ts.exported_bindings {
            if export.specifier.is_some() || exported_value(ts, &export.local) {
                continue;
            }
            let Some(offset) = export_offset(source, &export.local) else {
                continue;
            };
            emit(
                &mut findings,
                root,
                &ctx,
                offset,
                &name_fragments(&ctx, &export.local),
            );
        }
    }
    findings.sort();
    findings.dedup();
    findings
}

fn exported_local(ts: &crate::codebase::ts_source::facts::TsFileFacts, name: &str) -> bool {
    ts.exported_bindings
        .iter()
        .any(|export| export.specifier.is_none() && export.local == name)
}

fn exported_value(ts: &crate::codebase::ts_source::facts::TsFileFacts, name: &str) -> bool {
    ts.value_flow
        .bindings
        .iter()
        .any(|binding| binding.owner.is_none() && binding.name == name)
}

fn emit(
    findings: &mut Vec<RuleFinding>,
    root: &Path,
    ctx: &FileCtx<'_>,
    offset: u32,
    fragments: &[Fragment],
) {
    for fragment in fragments {
        let (message, import, target) = if let Some(specifier) = &fragment.open_specifier {
            (
                format!(
                    "unresolved import `{specifier}` may write, rebind, or escape an exported SQL fragment"
                ),
                specifier.clone(),
                specifier.clone(),
            )
        } else {
            let producer = relative_slash_path(root, &fragment.producer);
            let message = format!(
                "exported SQL fragment `{}` from `{producer}` is written, rebound, or escaped",
                fragment.exported
            );
            (message, producer, fragment.exported.clone())
        };
        findings.push(RuleFinding {
            causes: Vec::new(),
            rule: RULE_ID.to_string(),
            file: relative_slash_path(root, ctx.file),
            line: line_number(ctx.source, offset),
            message,
            import: Some(import),
            target: Some(target),
        });
    }
}

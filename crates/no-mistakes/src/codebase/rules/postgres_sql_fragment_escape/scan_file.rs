use super::fact_reads::export_site;
use super::fragments::Fragment;
use super::resolve::{binding_fragments, call_fragments, name_fragments, FileCtx};
use super::scan::ScanReport;
use super::source_text::file_text;
use super::RULE_ID;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::check_outcome::CheckFailure;
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use crate::codebase::postgres::embedded::TrustedSqlTag;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::{line_number, relative_slash_path};
use crate::fx::fx_map;
use std::path::Path;

pub(super) fn one(
    root: &Path,
    tags: &[TrustedSqlTag],
    graph: &DepGraph,
    facts: &CheckFactMap,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
    catalog: &super::fragments::Catalog<'_>,
    file: &Path,
) -> Option<ScanReport> {
    let ts = facts.get_ts_facts(file)?;
    let mut gaps = Vec::new();
    let relative = relative_slash_path(root, file);
    if let Some(reason) = &ts.operational_error {
        gaps.push(gap(
            file,
            format!(
                "`{relative}` has an operational error ({reason}), so a write, rebind, or escape of an exported SQL fragment may be missing. Fix that error and rerun the check."
            ),
        ));
    }
    if ts.fatal_parse_error {
        gaps.push(gap(
            file,
            format!(
                "`{relative}` has a fatal parse error, so a write, rebind, or escape of an exported SQL fragment may be missing. Fix the syntax error and rerun the check."
            ),
        ));
    }
    if let Some(reason) = &ts.value_flow.incomplete_reason {
        gaps.push(gap(
            file,
            format!(
                "`{relative}` value flow is incomplete ({reason}), so a write, rebind, or escape of an exported SQL fragment may be missing. Simplify the file so value flow finishes, then rerun the check."
            ),
        ));
    }
    let owned = file_text(ts, sources, file);
    if owned.is_none() {
        gaps.push(gap(
            file,
            format!(
                "`{relative}` could not be read, so a write, rebind, or escape of an exported SQL fragment in this file may be missing. Restore a readable source file and rerun the check."
            ),
        ));
    }
    let source = owned.as_deref().unwrap_or("");
    let mut findings = Vec::new();
    let empty = fx_map();
    let probe = FileCtx {
        graph,
        facts,
        catalog,
        tags,
        file,
        ts,
        source,
        callbacks: &empty,
        context: None,
    };
    let callbacks = super::dynamic::callback_modules(&probe);
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
        let Some(offset) = export_site(ts, source, &export.local) else {
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
    Some(ScanReport { findings, gaps })
}

fn gap(file: &Path, message: String) -> CheckFailure {
    CheckFailure {
        consumer: RULE_ID.to_string(),
        file: Some(file.to_path_buf()),
        property: None,
        message,
    }
}

fn exported_local(ts: &TsFileFacts, name: &str) -> bool {
    ts.exported_bindings
        .iter()
        .any(|export| export.specifier.is_none() && export.local == name)
}

fn exported_value(ts: &TsFileFacts, name: &str) -> bool {
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

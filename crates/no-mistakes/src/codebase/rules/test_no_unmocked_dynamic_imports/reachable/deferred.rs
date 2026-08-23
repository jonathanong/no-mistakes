use super::super::checker::{evaluate_dynamic_import, DynamicCheckContext};
use super::super::{runtime_deps, RULE_ID};
use super::{collect_outcome, get_or_cache_file, is_under_skipped_dir};
use super::{ReachableContext, ReachableResult};
use crate::codebase::ts_source::{has_disable_comment, has_disable_file_comment};
use anyhow::Result;
use dashmap::DashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn collect(
    ctx: ReachableContext<'_>,
    test_file: &Path,
    mocks: &HashSet<PathBuf>,
    dependency_cache: &DashMap<PathBuf, Arc<Vec<PathBuf>>>,
    defer_suppression: bool,
) -> Result<ReachableResult> {
    let test_reachable = dependency_cache
        .entry(test_file.to_path_buf())
        .or_insert_with(|| {
            Arc::new(runtime_deps(
                ctx.graph,
                test_file.to_path_buf(),
                ctx.file_universe,
            ))
        })
        .clone();
    let mut result = ReachableResult {
        findings: Vec::new(),
        covered: HashSet::new(),
    };
    for file in test_reachable.iter() {
        if !crate::codebase::dependencies::extract::is_indexable(file)
            || is_under_skipped_dir(ctx.root, ctx.config, file)
        {
            continue;
        }
        // Mock factories replace the target body, so its own imports are not
        // reachable. This also prevents typed mock carriers from leaking.
        if mocks.contains(file) {
            continue;
        }
        if let Some(shared) = ctx.shared {
            let Some(file_facts) = shared.ts.get(file) else {
                continue;
            };
            if file_facts.parse_error.is_some() {
                continue;
            }
            // A prepared request is authoritative, including incomplete or
            // failed entries. Falling back to disk would violate one-pass
            // ownership and can produce findings from facts outside the
            // request's declared inventory.
            let Some(source) = file_facts.source.as_deref() else {
                continue;
            };
            let Some(facts) = file_facts.dynamic_imports.as_ref() else {
                continue;
            };
            if !defer_suppression && has_disable_file_comment(source, RULE_ID) {
                continue;
            }
            let mut local_findings = Vec::new();
            let check_context = DynamicCheckContext {
                root: ctx.root,
                file,
                resolver: ctx.resolver,
                graph: ctx.graph,
                graph_files: ctx.graph_files,
                file_universe: ctx.file_universe,
                mocks,
                dependency_cache,
                findings: &mut local_findings,
            };
            for import in &facts.dynamic_imports {
                if defer_suppression || !has_disable_comment(source, import.line as u32, RULE_ID) {
                    collect_outcome(
                        &mut result,
                        evaluate_dynamic_import(&check_context, import.clone()),
                    );
                }
            }
            continue;
        }
        let cached = get_or_cache_file(file, ctx.file_cache)?;
        if !defer_suppression && has_disable_file_comment(&cached.source, RULE_ID) {
            continue;
        }
        let mut local_findings = Vec::new();
        let check_context = DynamicCheckContext {
            root: ctx.root,
            file,
            resolver: ctx.resolver,
            graph: ctx.graph,
            graph_files: ctx.graph_files,
            file_universe: ctx.file_universe,
            mocks,
            dependency_cache,
            findings: &mut local_findings,
        };
        for import in &cached.dynamic_imports {
            if !defer_suppression
                && has_disable_comment(&cached.source, import.line as u32, RULE_ID)
            {
                continue;
            }
            collect_outcome(
                &mut result,
                evaluate_dynamic_import(&check_context, import.clone()),
            );
        }
    }
    Ok(result)
}

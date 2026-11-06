mod append;

use super::super::RULE_ID;
use crate::codebase::postgres::statements::SqlBoundFact;
use crate::codebase::postgres::AllowList;
use crate::codebase::postgres::SqlStatementFileFacts;
use crate::codebase::rules::{LocatedRuleFinding, PostgresFindings};
use crate::codebase::ts_source::{matching_disable_directive, SourceStore};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

type StatementKey = (PathBuf, Option<usize>, (usize, usize));

#[derive(Default)]
pub(super) struct Groups(BTreeMap<StatementKey, Group>);

#[derive(Default)]
struct Group {
    findings: PostgresFindings,
    executor: PathBuf,
    call_line: Option<usize>,
    spans: Vec<append::Span>,
}

impl Groups {
    pub(super) fn collect(
        &mut self,
        output: &mut PostgresFindings,
        start: usize,
        file: &SqlStatementFileFacts,
        bound_index: usize,
        bound: &SqlBoundFact,
    ) {
        let call = file
            .variant_locations
            .as_ref()
            .map(|locations| locations.call_index);
        let at = bound.statement_start.unwrap_or((bound.line, bound.column));
        let group = self.0.entry((file.path.clone(), call, at)).or_default();
        group.executor.clone_from(&file.path);
        group.call_line = file
            .variant_locations
            .as_ref()
            .map(|locations| locations.original_call_line as usize);
        if let Some(span) = append::span(file, bound_index) {
            group.spans.push(span);
        }
        group.findings.0.extend(output.0.drain(start..));
    }

    pub(super) fn finish(
        self,
        output: &mut PostgresFindings,
        root: &Path,
        sources: &SourceStore,
        allow: &AllowList,
    ) {
        for (_, group) in self.0 {
            let executor = crate::codebase::ts_source::relative_slash_path(root, &group.executor);
            let mut findings = group.findings;
            merge(
                &mut findings,
                (&executor, group.call_line),
                &group.spans,
                root,
                sources,
                allow,
            );
            output.extend(findings);
        }
    }
}

fn merge(
    output: &mut PostgresFindings,
    (executor, call_line): (&str, Option<usize>),
    spans: &[append::Span],
    root: &Path,
    sources: &SourceStore,
    allow: &AllowList,
) {
    if output.0.is_empty() {
        return;
    }
    let mut group = Vec::new();
    let candidates = std::mem::take(&mut output.0);
    for located in candidates {
        let finding = &located.finding;
        let source = crate::codebase::rules::read_source(sources, &root.join(&finding.file));
        let disabled = source.as_deref().is_some_and(|source| {
            matching_disable_directive(source, u32::try_from(finding.line).ok(), RULE_ID).is_some()
        });
        if disabled
            || finding
                .target
                .as_deref()
                .is_some_and(|target| allow.contains(target))
        {
            output.0.push(located);
        } else {
            group.push(located);
        }
    }
    group.sort();
    group.sort_by_key(|located| located.finding.file != executor);
    let appended_call = append::only_appended(spans);
    let mut roots = BTreeMap::<String, LocatedRuleFinding>::new();
    for mut located in group {
        let Some(target) = located.finding.target.clone() else {
            output.0.push(located);
            continue;
        };
        let original = &located.finding;
        let imported = original.file != executor;
        let relocation = if imported {
            call_line
        } else {
            appended_call
                .or_else(|| append::relocate(spans, std::slice::from_ref(&located), executor))
        };
        if imported || relocation.is_some() {
            let file = original.file.clone();
            let line = original.line;
            let suffix = original
                .message
                .strip_prefix(&format!("{file}:{line}: "))
                .unwrap_or(&original.message)
                .to_string();
            let fragment_line = if imported {
                Some(line)
            } else {
                append::fragment_line(spans, &target)
            };
            let note = fragment_line
                .map(|line| format!(" (includes SQL appended from {file}:{line})"))
                .unwrap_or_default();
            located.finding.file = executor.to_string();
            located.finding.line = relocation.unwrap_or(line).max(1);
            located.finding.message = format!(
                "{}:{}: {suffix}{note}",
                located.finding.file, located.finding.line
            );
        }
        roots.entry(target).or_insert(located);
    }
    output.0.extend(roots.into_values());
}

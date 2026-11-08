mod append;
mod normalize;
mod report;
mod site;

use super::super::RULE_ID;
use crate::codebase::postgres::statements::SqlBoundFact;
use crate::codebase::postgres::AllowList;
use crate::codebase::postgres::SqlStatementFileFacts;
use crate::codebase::rules::{LocatedRuleFinding, PostgresFindings};
use crate::codebase::ts_source::{matching_disable_directive, SourceStore};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Execution {
    Call(usize),
    Projection(usize),
}

type StatementKey = (PathBuf, Option<Execution>, site::Position);

#[derive(Default)]
pub(super) struct Groups {
    statements: BTreeMap<StatementKey, Group>,
    positions: site::Positions,
}

#[derive(Default)]
struct Group {
    findings: PostgresFindings,
    executor: PathBuf,
    call_line: Option<usize>,
    spans: Vec<append::Span>,
    at: (usize, usize),
}

impl Groups {
    pub(super) fn collect(
        &mut self,
        output: &mut PostgresFindings,
        start: usize,
        file: &SqlStatementFileFacts,
        bound_index: usize,
        bound: &SqlBoundFact,
        embedded_projection: Option<usize>,
    ) {
        let call = file
            .variant_locations
            .as_ref()
            .map(|locations| Execution::Call(locations.call_index))
            .or_else(|| embedded_projection.map(Execution::Projection));
        let (position, at) =
            self.positions
                .identify(file, bound_index, bound, embedded_projection, call);
        let group = self
            .statements
            .entry((file.path.clone(), call, position))
            .or_insert_with(|| Group {
                at,
                ..Default::default()
            });
        group.at = group.at.min(at);
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
        let mut sites = Vec::new();
        for (_, group) in normalize::coalesce(self.statements, self.positions) {
            let executor = crate::codebase::ts_source::relative_slash_path(root, &group.executor);
            let mut findings = group.findings;
            let grouped = merge(
                &mut findings,
                (&executor, group.call_line),
                &group.spans,
                root,
                sources,
                allow,
            );
            output.extend(findings);
            if let Some(grouped) = grouped {
                sites.push((output.len(), group.at));
                output.0.push(grouped);
            }
        }
        report::disambiguate(output, &sites);
    }
}

fn merge(
    output: &mut PostgresFindings,
    (executor, call_line): (&str, Option<usize>),
    spans: &[append::Span],
    root: &Path,
    sources: &SourceStore,
    allow: &AllowList,
) -> Option<LocatedRuleFinding> {
    if output.0.is_empty() {
        return None;
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
    report::combine(roots)
}

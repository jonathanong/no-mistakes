mod append;

use super::super::RULE_ID;
use crate::codebase::postgres::statements::SqlBoundFact;
use crate::codebase::postgres::AllowList;
use crate::codebase::postgres::SqlStatementFileFacts;
use crate::codebase::rules::{LocatedRuleFinding, PostgresFindings};
use crate::codebase::ts_source::{matching_disable_directive, SourceStore};
use std::collections::{BTreeMap, BTreeSet};
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
            let relocate_line = append::relocate(&group.spans, &group.findings.0, &executor);
            let mut findings = group.findings;
            merge(
                &mut findings,
                (&executor, group.call_line, relocate_line),
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
    (executor, call_line, relocate_line): (&str, Option<usize>, Option<usize>),
    root: &Path,
    sources: &SourceStore,
    allow: &AllowList,
) {
    if output.len() < 2 {
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
    if group.len() < 2 {
        output.0.extend(group);
        return;
    }
    group.sort();
    let roots: BTreeSet<_> = group
        .iter()
        .filter_map(|located| located.finding.target.clone())
        .collect();
    let targets = roots.into_iter().collect::<Vec<_>>().join(", ");
    let mut fragments: BTreeSet<_> = group
        .iter()
        .filter(|located| located.finding.file != executor)
        .map(|located| format!("{}:{}", located.finding.file, located.finding.line))
        .collect();
    let local = group
        .iter()
        .position(|located| located.finding.file == executor);
    let relocate = relocate_line.filter(|_| local.is_some());
    if relocate.is_some() {
        for located in &group {
            if located.finding.file == executor {
                fragments.insert(format!("{}:{}", located.finding.file, located.finding.line));
            }
        }
    }
    let mut located: LocatedRuleFinding = group.remove(local.unwrap_or(0));
    let finding = &mut located.finding;
    if local.is_none() {
        finding.file = executor.to_string();
        finding.line = call_line.unwrap_or(finding.line).max(1);
    } else if let Some(line) = relocate {
        finding.line = line.max(1);
    }
    if let Some(line) = relocate {
        fragments.remove(&format!("{}:{}", finding.file, line));
    }
    finding.target = Some(targets.clone());
    let note = if fragments.is_empty() {
        String::new()
    } else {
        let fragments = fragments.into_iter().collect::<Vec<_>>().join(", ");
        format!(" (includes SQL appended from {fragments})")
    };
    finding.message = format!(
        "{}:{}: statement can scan unbounded rows from roots {targets}{note}; add a row cap, match complete unique keys or declared bounded columns, or allow each root with a reason",
        finding.file, finding.line
    );
    output.0.push(located);
}

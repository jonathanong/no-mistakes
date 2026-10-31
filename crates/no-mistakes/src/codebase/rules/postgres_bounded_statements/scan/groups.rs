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
pub(super) struct Groups(BTreeMap<StatementKey, PostgresFindings>);

impl Groups {
    pub(super) fn collect(
        &mut self,
        output: &mut PostgresFindings,
        start: usize,
        file: &SqlStatementFileFacts,
        bound: &SqlBoundFact,
    ) {
        let call = file
            .variant_locations
            .as_ref()
            .map(|locations| locations.call_index);
        let at = bound.statement_start.unwrap_or((bound.line, bound.column));
        self.0
            .entry((file.path.clone(), call, at))
            .or_default()
            .0
            .extend(output.0.drain(start..));
    }

    pub(super) fn finish(
        self,
        output: &mut PostgresFindings,
        root: &Path,
        sources: &SourceStore,
        allow: &AllowList,
    ) {
        for (_, mut group) in self.0 {
            merge(&mut group, root, sources, allow);
            output.extend(group);
        }
    }
}

fn merge(output: &mut PostgresFindings, root: &Path, sources: &SourceStore, allow: &AllowList) {
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
    let mut located: LocatedRuleFinding = group.remove(0);
    let finding = &mut located.finding;
    finding.target = Some(targets.clone());
    finding.message = format!(
        "{}:{}: statement can scan unbounded rows from roots {targets}; add a row cap, match complete unique keys or declared bounded columns, or allow each root with a reason",
        finding.file, finding.line
    );
    output.0.push(located);
}

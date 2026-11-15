use super::groups::Groups;
use super::messages::{finding, message};
use crate::codebase::postgres::{SqlBoundFact, SqlBoundKind, SqlStatementFileFacts};
use crate::codebase::rules::{PostgresFindings, VariantFindingDedup};

pub(super) struct Offenders<'a, 'src> {
    pub(super) output: &'a mut PostgresFindings,
    pub(super) groups: &'a mut Groups,
    pub(super) attribution: &'a mut VariantFindingDedup<'src>,
    pub(super) facts: &'a SqlStatementFileFacts,
    pub(super) source: Option<&'a str>,
    pub(super) rel: &'a str,
    pub(super) bound_index: usize,
    pub(super) bound: &'a SqlBoundFact,
    pub(super) file_index: usize,
    pub(super) native_statement_count: usize,
    pub(super) offenders: Vec<super::super::evaluate::Offender>,
    pub(super) domain_rows: Option<u64>,
}

pub(super) fn offenders(input: Offenders<'_, '_>) {
    let Offenders {
        output,
        groups,
        attribution,
        facts,
        source,
        rel,
        bound_index,
        bound,
        file_index,
        native_statement_count,
        offenders,
        domain_rows,
    } = input;
    let group_start = output.len();
    for offender in offenders {
        // Keep suppression and its audit in the shared layer. A directive
        // on the statement start anchors all of that statement's findings.
        let line = bound
            .statement_start
            .and_then(|(line, _)| super::statement_directive_line(source, line))
            .or_else(|| super::statement_directive_line(source, bound.line))
            .unwrap_or(if bound.body.is_some() {
                bound.line
            } else {
                offender.line
            })
            .max(1);
        let body_message = bound
            .body
            .as_ref()
            .map(|body| super::super::bodies::message(body, &offender.table));
        let site = facts
            .variant_locations
            .as_ref()
            .filter(|_| bound.body.is_none())
            .and_then(|locations| {
                locations
                    .bound_table_sites(bound_index, &offender.table, offender.line)
                    .find(|(_, position)| position.sql_column == offender.column)
                    .map(|(site, _)| site)
            })
            .unwrap_or(crate::codebase::postgres::statements::SqlFactSite::Bound(
                bound_index,
            ));
        attribution.push_attributed(
            output,
            facts,
            site,
            source,
            finding(
                rel,
                line,
                &body_message.unwrap_or_else(|| {
                    let kind = if bound.target.is_none() {
                        SqlBoundKind::Select
                    } else {
                        bound.kind
                    };
                    message(
                        kind,
                        &offender.table,
                        &offender.object,
                        offender.domain_over,
                        domain_rows,
                    )
                }),
                &offender.object.to_string(),
            ),
        );
    }
    groups.collect(
        output,
        group_start,
        facts,
        bound_index,
        bound,
        (file_index >= native_statement_count).then_some(file_index),
    );
}

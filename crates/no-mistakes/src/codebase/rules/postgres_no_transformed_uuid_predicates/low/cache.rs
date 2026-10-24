use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::postgres::sql_identifier_syntax::helper_recovery::ApplicationKey;
use crate::codebase::postgres::sql_identifier_syntax::literal_projection::{
    project_literals, LiteralProjection,
};
use crate::codebase::postgres::sql_identifier_syntax::TransformedLiteral;
use std::collections::BTreeMap;
use std::path::Path;

type Span = (Option<ApplicationKey>, (u32, u32));
pub(super) fn literals<'a>(
    file: &'a CheckFileFacts,
    facts: &CheckFactMap,
    path: &Path,
    source: &str,
) -> &'a BTreeMap<Span, TransformedLiteral> {
    file.sql_identifier_syntax.transformed_literals.get_or_init(|| project_literals(
        file, facts, path, source,
        LiteralProjection {
            project: |statement| statement.transformed_column_uses.clone(),
            parse: |text| {
                let statement = crate::codebase::postgres::statements::extract_sql_statement_facts_with_bounds(text, false);
                if statement.parse_failed { Err(()) } else { Ok(statement.transformed_column_uses) }
            },
            rebase: |fact, base| fact.line += base - 1,
            position: |fact| (fact.line, fact.column_position),
            parse_metric: "postgres.no_transformed_uuid_predicates.low.literal_parses",
        },
    ))
}

pub(super) fn recovered_finding(
    root: &Path,
    caller: &Path,
    sql: &crate::codebase::postgres::RecoveredSql,
    use_: &crate::codebase::postgres::statements::SqlTransformedColumnUseFact,
    table: &str,
) -> crate::codebase::rules::RuleFinding {
    let (origin, line) =
        crate::codebase::postgres::sql_identifier_syntax::helper_recovery::Recovered::position(
            sql,
            use_.line,
            use_.column_position,
            caller,
        );
    let origin_rel = crate::codebase::ts_source::relative_slash_path(root, origin);
    let mut relocated = use_.clone();
    relocated.line = line;
    let finding = super::super::scan::finding(&origin_rel, table, &relocated);
    finding
}

pub(super) fn mark_uncertain(
    status: &mut crate::codebase::postgres::SqlVerdictStatus,
    reasons: &mut Vec<String>,
    invalid: bool,
) {
    if *status != crate::codebase::postgres::SqlVerdictStatus::Violation {
        *status = crate::codebase::postgres::SqlVerdictStatus::Abstain;
    }
    reasons.push(if invalid {
        "invalid-sql".into()
    } else {
        "SQL helper recovery has residual uncertainty".into()
    });
}

pub(super) fn mark_input_unknown(
    status: &mut crate::codebase::postgres::SqlVerdictStatus,
    uncertain: &mut bool,
    reasons: &mut Vec<String>,
) {
    if *status != crate::codebase::postgres::SqlVerdictStatus::Violation {
        *status = crate::codebase::postgres::SqlVerdictStatus::Abstain;
    }
    *uncertain = true;
    reasons.push("SQL is unanalyzable at effort low; pass a literal string or template directly to the imported executor, or through one local const; use effort max for inferred SQL".into());
}

use super::super::{Input, Span};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::SqlStatementFileFacts;
use std::{collections::BTreeMap, sync::Arc};

pub(crate) type RecoveredStatements = BTreeMap<
    (super::super::helper_recovery::ApplicationKey, Span),
    Vec<Result<Arc<SqlStatementFileFacts>, ()>>,
>;

/// Parse each recovered alternative once for all low SQL policies in this request.
pub(crate) fn recovered_statements(file: &CheckFileFacts) -> &RecoveredStatements {
    file.sql_identifier_syntax
        .recovered_statements
        .get_or_init(|| {
            let mut recovered = BTreeMap::new();
            let mut parsed = BTreeMap::<String, Result<Arc<SqlStatementFileFacts>, ()>>::new();
            for application in file.sql_identifier_syntax.application_keys() {
                let Some(calls) = file.sql_identifier_syntax.calls_for(&application) else {
                    continue;
                };
                for call in calls {
                    let Input::Recovered(value) = &call.input else {
                        continue;
                    };
                    recovered
                        .entry((application.clone(), call.span))
                        .or_insert_with(|| {
                            value
                                .alternatives
                                .iter()
                                .map(|sql| {
                                    parsed
                                        .entry(sql.sql.clone())
                                        .or_insert_with(|| {
                                            if let Some(observer) = crate::diagnostics::current() {
                                                observer
                                                    .increment("postgres.low.recovered_parses", 1);
                                            }
                                            let facts = crate::codebase::postgres::statements::
                                                extract_sql_statement_facts_with_bounds(
                                                    &sql.sql, false,
                                                );
                                            if facts.parse_failed {
                                                Err(())
                                            } else {
                                                Ok(Arc::new(facts))
                                            }
                                        })
                                        .clone()
                                })
                                .collect()
                        });
                }
            }
            recovered
        })
}

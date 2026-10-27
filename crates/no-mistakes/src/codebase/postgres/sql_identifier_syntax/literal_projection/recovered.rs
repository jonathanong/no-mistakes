use super::super::{Input, Span};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::SqlStatementFileFacts;
use std::{collections::BTreeMap, sync::Arc};

type Key = (super::super::helper_recovery::ApplicationKey, Span);
pub(crate) type RecoveredStatements = BTreeMap<Key, Vec<Result<Arc<SqlStatementFileFacts>, ()>>>;
pub(crate) type RawStatements = BTreeMap<Key, Vec<Arc<SqlStatementFileFacts>>>;

/// Preserve partial facts for policy findings; invalid SQL never becomes a clear proof.
pub(crate) fn raw_statements(file: &CheckFileFacts) -> &RawStatements {
    file.sql_identifier_syntax.recovered_raw.get_or_init(|| {
        let mut recovered = BTreeMap::new();
        let mut parsed = BTreeMap::<String, Arc<SqlStatementFileFacts>>::new();
        for application in file.sql_identifier_syntax.application_keys() {
            let Some(calls) = file.sql_identifier_syntax.calls_for(&application) else { continue; };
            for call in calls {
                let Input::Recovered(value) = &call.input else { continue; };
                recovered.entry((application.clone(), call.span)).or_insert_with(|| {
                    value.alternatives.iter().map(|sql| {
                        parsed.entry(sql.sql.clone()).or_insert_with(|| {
                            if let Some(observer) = crate::diagnostics::current() {
                                observer.increment("postgres.low.recovered_parses", 1);
                            }
                            let facts = crate::codebase::postgres::statements::extract_sql_statement_facts_with_location_demand(
                                &sql.sql, false, &[], crate::codebase::postgres::statement_facts::SqlLocationDemand::Full);
                            Arc::new(facts)
                        }).clone()
                    }).collect()
                });
            }
        }
        recovered
    })
}

/// The older specialized policies retain their invalid-input contract.
pub(crate) fn recovered_statements(file: &CheckFileFacts) -> &RecoveredStatements {
    file.sql_identifier_syntax
        .recovered_statements
        .get_or_init(|| {
            if !rich(file) {
                return legacy(file);
            }
            raw_statements(file)
                .iter()
                .map(|(key, statements)| {
                    (
                        key.clone(),
                        statements
                            .iter()
                            .map(|facts| {
                                if facts.parse_failed {
                                    Err(())
                                } else {
                                    Ok(Arc::clone(facts))
                                }
                            })
                            .collect(),
                    )
                })
                .collect()
        })
}

fn rich(file: &CheckFileFacts) -> bool {
    file.sql_identifier_syntax
        .low_applications
        .iter()
        .any(|(_, rule, _)| super::super::statement_policy::supports(rule))
}

// Keep the original specialized-only memory shape: no raw map or retained invalid payloads.
fn legacy(file: &CheckFileFacts) -> RecoveredStatements {
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
            recovered.entry((application.clone(), call.span)).or_insert_with(|| {
                value.alternatives.iter().map(|sql| {
                    parsed.entry(sql.sql.clone()).or_insert_with(|| {
                        if let Some(observer) = crate::diagnostics::current() {
                            observer.increment("postgres.low.recovered_parses", 1);
                        }
                        let facts = crate::codebase::postgres::statements::extract_sql_statement_facts_with_bounds(&sql.sql, false);
                        if facts.parse_failed { Err(()) } else { Ok(Arc::new(facts)) }
                    }).clone()
                }).collect()
            });
        }
    }
    recovered
}

//! Project writes from complete standalone programs using the prepared AST.
mod literals;
mod settings;
mod transaction;

use crate::codebase::postgres::{source, PostgresSqlExecution};
use literals::direct_write_kind;
use settings::{query_set_config_effects, set_statement_effect, start_transaction_can_commit};
use sqlparser::{
    ast::{DescribeAlias, Statement},
    tokenizer::Location,
};
use transaction::{apply_effect, Transaction};

#[derive(Debug)]
pub(crate) struct WritePlan {
    pub(crate) location: Location,
    pub(crate) kind: &'static str,
    pub(crate) autocommit: bool,
}

#[derive(Debug)]
pub(crate) struct UnanalyzablePlan {
    pub(crate) location: Location,
    pub(crate) reason: &'static str,
}

#[derive(Debug, Default)]
pub(crate) struct ExplainWriteRollbackFile {
    pub(crate) writes: Vec<WritePlan>,
    pub(crate) unanalyzable: Vec<UnanalyzablePlan>,
}

pub(super) fn unparseable() -> ExplainWriteRollbackFile {
    unanalyzable_file("SQL script could not be parsed completely")
}

pub(super) fn partial_program() -> ExplainWriteRollbackFile {
    unanalyzable_file("SQL script ended at an incomplete statement boundary")
}

fn unanalyzable_file(reason: &'static str) -> ExplainWriteRollbackFile {
    ExplainWriteRollbackFile {
        writes: Vec::new(),
        unanalyzable: vec![UnanalyzablePlan {
            location: Location::new(1, 1),
            reason,
        }],
    }
}

pub(super) fn collect(
    statements: &[Statement],
    locations: &[Location],
) -> ExplainWriteRollbackFile {
    let mut out = ExplainWriteRollbackFile::default();
    let mut session_read_only = false;
    let mut transaction: Option<Transaction> = None;
    for (statement, &location) in statements.iter().zip(locations) {
        match statement {
            Statement::StartTransaction { modes, .. } => {
                transaction.get_or_insert_with(|| {
                    Transaction::new(
                        start_transaction_can_commit(modes, session_read_only),
                        session_read_only,
                    )
                });
            }
            Statement::Commit { chain, .. }
            | Statement::Rollback {
                chain,
                savepoint: None,
            } => {
                if let Some(active) = transaction.take() {
                    if matches!(statement, Statement::Commit { .. }) {
                        out.writes.extend(active.pending);
                    } else {
                        session_read_only = active.saved_session_read_only;
                    }
                    if *chain {
                        transaction = Some(Transaction::new(
                            active.can_commit_writes,
                            session_read_only,
                        ));
                    }
                }
            }
            Statement::Savepoint { name } => {
                if let Some(active) = &mut transaction {
                    active.savepoint(name, session_read_only);
                }
            }
            Statement::ReleaseSavepoint { name } => {
                if let Some(active) = &mut transaction {
                    active.release(name, location, &mut out);
                }
            }
            Statement::Rollback {
                savepoint: Some(name),
                ..
            } => {
                if let Some(active) = &mut transaction {
                    active.rollback_to(name, location, &mut session_read_only, &mut out);
                }
            }
            Statement::Set(set) => {
                if let Some(effect) = set_statement_effect(set) {
                    apply_effect(
                        effect,
                        location,
                        &mut session_read_only,
                        &mut transaction,
                        &mut out,
                    );
                }
            }
            Statement::Query(query) => {
                for effect in query_set_config_effects(query) {
                    apply_effect(
                        effect,
                        location,
                        &mut session_read_only,
                        &mut transaction,
                        &mut out,
                    );
                }
            }
            Statement::Explain {
                describe_alias: DescribeAlias::Explain,
                analyze,
                query_plan,
                estimate,
                format,
                options,
                statement,
                ..
            } => {
                let Some(kind) = direct_write_kind(statement) else {
                    continue;
                };
                match source::explain_execution(
                    *analyze,
                    *query_plan,
                    *estimate,
                    *format,
                    options.as_deref(),
                ) {
                    PostgresSqlExecution::ExecutesForAnalysis => {
                        record_write(
                            &mut transaction,
                            session_read_only,
                            location,
                            kind,
                            &mut out,
                        );
                    }
                    PostgresSqlExecution::Unknown => {
                        out.unanalyzable.push(UnanalyzablePlan {
                            location,
                            reason: "EXPLAIN options could not be statically evaluated",
                        });
                    }
                    PostgresSqlExecution::NonExecuting => {}
                }
            }
            _ => {}
        }
    }
    out
}

fn record_write(
    transaction: &mut Option<Transaction>,
    session_read_only: bool,
    location: Location,
    kind: &'static str,
    out: &mut ExplainWriteRollbackFile,
) {
    match transaction {
        Some(active) if active.can_commit_writes => active.pending.push(WritePlan {
            location,
            kind,
            autocommit: false,
        }),
        Some(_) => {}
        None if !session_read_only => out.writes.push(WritePlan {
            location,
            kind,
            autocommit: true,
        }),
        None => {}
    }
}

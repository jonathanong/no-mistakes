//! Project writes from complete standalone programs using the prepared AST.
mod literals;
mod settings;

use crate::codebase::postgres::{source, PostgresSqlExecution};
use literals::{direct_write_kind, normalize_ident};
use settings::{
    query_set_config_effects, set_statement_effect, start_transaction_can_commit, ReadOnlySetting,
    SettingEffect,
};
use sqlparser::{
    ast::{DescribeAlias, Ident, Statement},
    tokenizer::Location,
};

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

struct Transaction {
    can_commit_writes: bool,
    saved_session_read_only: bool,
    pending: Vec<WritePlan>,
    savepoints: Vec<(String, usize)>,
}

impl Transaction {
    fn new(can_commit_writes: bool, saved_session_read_only: bool) -> Self {
        Self { can_commit_writes, saved_session_read_only, pending: Vec::new(), savepoints: Vec::new() }
    }

    fn release(&mut self, name: &Ident, location: Location, out: &mut ExplainWriteRollbackFile) {
        let key = normalize_ident(name);
        if let Some(pos) = self.savepoints.iter().rposition(|(sp, _)| sp == &key) {
            self.savepoints.truncate(pos);
        } else {
            out.unanalyzable.push(UnanalyzablePlan { location, reason: "RELEASE SAVEPOINT targets an unknown savepoint" });
        }
    }

    fn rollback_to(&mut self, name: &Ident, location: Location, out: &mut ExplainWriteRollbackFile) {
        let key = normalize_ident(name);
        if let Some(pos) = self.savepoints.iter().rposition(|(sp, _)| sp == &key) {
            self.pending.truncate(self.savepoints[pos].1);
            self.savepoints.truncate(pos + 1);
        } else {
            out.unanalyzable.push(UnanalyzablePlan { location, reason: "ROLLBACK TO SAVEPOINT targets an unknown savepoint" });
        }
    }
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
        unanalyzable: vec![UnanalyzablePlan { location: Location::new(1, 1), reason }],
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
            Statement::Commit { chain, .. } | Statement::Rollback { chain, savepoint: None } => {
                if let Some(active) = transaction.take() {
                    if matches!(statement, Statement::Commit { .. }) {
                        out.writes.extend(active.pending);
                    } else {
                        session_read_only = active.saved_session_read_only;
                    }
                    if *chain {
                        transaction = Some(Transaction::new(active.can_commit_writes, session_read_only));
                    }
                }
            }
            Statement::Savepoint { name } => {
                if let Some(active) = &mut transaction {
                    active.savepoints.push((normalize_ident(name), active.pending.len()));
                }
            }
            Statement::ReleaseSavepoint { name } => {
                if let Some(active) = &mut transaction {
                    active.release(name, location, &mut out);
                }
            }
            Statement::Rollback { savepoint: Some(name), .. } => {
                if let Some(active) = &mut transaction {
                    active.rollback_to(name, location, &mut out);
                }
            }
            Statement::Set(set) => {
                if let Some(effect) = set_statement_effect(set) {
                    apply_effect(effect, location, &mut session_read_only, &mut transaction, &mut out);
                }
            }
            Statement::Query(query) => {
                for effect in query_set_config_effects(query) {
                    apply_effect(effect, location, &mut session_read_only, &mut transaction, &mut out);
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
            } if source::explain_execution(*analyze, *query_plan, *estimate, *format, options.as_deref())
                == PostgresSqlExecution::ExecutesForAnalysis =>
            {
                if let Some(kind) = direct_write_kind(statement) {
                    match &mut transaction {
                        Some(active) if active.can_commit_writes => {
                            active.pending.push(WritePlan { location, kind, autocommit: false });
                        }
                        Some(_) => {}
                        None if !session_read_only => {
                            out.writes.push(WritePlan { location, kind, autocommit: true });
                        }
                        None => {}
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn apply_effect(
    effect: SettingEffect,
    location: Location,
    session_read_only: &mut bool,
    transaction: &mut Option<Transaction>,
    out: &mut ExplainWriteRollbackFile,
) {
    match effect {
        SettingEffect::Change { setting: ReadOnlySetting::DefaultTransaction, read_only, is_local: false } => {
            *session_read_only = read_only;
        }
        SettingEffect::Change { setting: ReadOnlySetting::DefaultTransaction, is_local: true, .. } => {}
        SettingEffect::Change { setting: ReadOnlySetting::CurrentTransaction, read_only, .. } => {
            if let Some(active) = transaction {
                active.can_commit_writes = !read_only;
                if read_only {
                    active.pending.clear();
                }
            }
        }
        SettingEffect::Unanalyzable(reason) => {
            out.unanalyzable.push(UnanalyzablePlan { location, reason });
        }
    }
}

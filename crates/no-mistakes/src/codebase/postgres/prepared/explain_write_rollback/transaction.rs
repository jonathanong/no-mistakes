//! Transaction and savepoint state machine for EXPLAIN write rollback.
use super::settings::{ReadOnlySetting, SettingEffect};
use super::{ExplainWriteRollbackFile, UnanalyzablePlan, WritePlan};
use crate::codebase::postgres::idents::ident_key;
use sqlparser::{ast::Ident, tokenizer::Location};

struct Savepoint {
    name: String,
    pending_len: usize,
    can_commit_writes: bool,
    session_read_only: bool,
}

pub(super) struct Transaction {
    pub(super) can_commit_writes: bool,
    pub(super) saved_session_read_only: bool,
    pub(super) pending: Vec<WritePlan>,
    savepoints: Vec<Savepoint>,
}

impl Transaction {
    pub(super) fn new(can_commit_writes: bool, saved_session_read_only: bool) -> Self {
        Self {
            can_commit_writes,
            saved_session_read_only,
            pending: Vec::new(),
            savepoints: Vec::new(),
        }
    }

    pub(super) fn savepoint(&mut self, name: &Ident, session_read_only: bool) {
        self.savepoints.push(Savepoint {
            name: ident_key(name),
            pending_len: self.pending.len(),
            can_commit_writes: self.can_commit_writes,
            session_read_only,
        });
    }

    pub(super) fn release(
        &mut self,
        name: &Ident,
        location: Location,
        out: &mut ExplainWriteRollbackFile,
    ) {
        let key = ident_key(name);
        if let Some(pos) = self.savepoints.iter().rposition(|sp| sp.name == key) {
            self.savepoints.truncate(pos);
        } else {
            out.unanalyzable.push(UnanalyzablePlan {
                location,
                reason: "RELEASE SAVEPOINT targets an unknown savepoint",
            });
        }
    }

    pub(super) fn rollback_to(
        &mut self,
        name: &Ident,
        location: Location,
        session_read_only: &mut bool,
        out: &mut ExplainWriteRollbackFile,
    ) {
        let key = ident_key(name);
        if let Some(pos) = self.savepoints.iter().rposition(|sp| sp.name == key) {
            let saved = &self.savepoints[pos];
            self.pending.truncate(saved.pending_len);
            self.can_commit_writes = saved.can_commit_writes;
            *session_read_only = saved.session_read_only;
            self.savepoints.truncate(pos + 1);
        } else {
            out.unanalyzable.push(UnanalyzablePlan {
                location,
                reason: "ROLLBACK TO SAVEPOINT targets an unknown savepoint",
            });
        }
    }
}

pub(super) fn apply_effect(
    effect: SettingEffect,
    location: Location,
    session_read_only: &mut bool,
    transaction: &mut Option<Transaction>,
    out: &mut ExplainWriteRollbackFile,
) {
    match effect {
        SettingEffect::Change {
            setting: ReadOnlySetting::DefaultTransaction,
            read_only,
            is_local: false,
        } => {
            *session_read_only = read_only;
        }
        SettingEffect::Change {
            setting: ReadOnlySetting::DefaultTransaction,
            is_local: true,
            ..
        } => {}
        SettingEffect::Change {
            setting: ReadOnlySetting::CurrentTransaction,
            read_only,
            ..
        } => {
            if let Some(active) = transaction {
                active.can_commit_writes = !read_only;
            }
        }
        SettingEffect::Unanalyzable(reason) => {
            out.unanalyzable.push(UnanalyzablePlan { location, reason });
        }
    }
}

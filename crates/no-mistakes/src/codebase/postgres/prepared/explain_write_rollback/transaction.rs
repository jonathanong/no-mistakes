//! Transaction and savepoint state machine for EXPLAIN write rollback.
use super::literals::normalize_ident;
use super::settings::{ReadOnlySetting, SettingEffect};
use super::{ExplainWriteRollbackFile, UnanalyzablePlan, WritePlan};
use sqlparser::{ast::Ident, tokenizer::Location};

pub(super) struct Transaction {
    pub(super) can_commit_writes: bool,
    pub(super) saved_session_read_only: bool,
    pub(super) pending: Vec<WritePlan>,
    savepoints: Vec<(String, usize)>,
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

    pub(super) fn savepoint(&mut self, name: &Ident) {
        self.savepoints
            .push((normalize_ident(name), self.pending.len()));
    }

    pub(super) fn release(
        &mut self,
        name: &Ident,
        location: Location,
        out: &mut ExplainWriteRollbackFile,
    ) {
        let key = normalize_ident(name);
        if let Some(pos) = self.savepoints.iter().rposition(|(sp, _)| sp == &key) {
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
        out: &mut ExplainWriteRollbackFile,
    ) {
        let key = normalize_ident(name);
        if let Some(pos) = self.savepoints.iter().rposition(|(sp, _)| sp == &key) {
            self.pending.truncate(self.savepoints[pos].1);
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

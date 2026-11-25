//! Project writes from complete standalone programs using the prepared AST.
use crate::codebase::postgres::{source, PostgresSqlExecution};
use sqlparser::{
    ast::{
        DescribeAlias, ObjectNamePart, Set, SetExpr, Statement, TransactionAccessMode,
        TransactionMode,
    },
    tokenizer::Location,
};

#[derive(Debug)]
pub(crate) struct WritePlan {
    pub(crate) location: Location,
    pub(crate) kind: &'static str,
    pub(crate) autocommit: bool,
}

struct Transaction {
    can_commit_writes: bool,
    pending: Option<Vec<WritePlan>>,
}

impl Transaction {
    fn new(can_commit_writes: bool) -> Self {
        Self {
            can_commit_writes,
            pending: can_commit_writes.then(Vec::new),
        }
    }
}

pub(super) fn collect(statements: &[Statement], locations: &[Location]) -> Vec<WritePlan> {
    let mut findings = Vec::new();
    let mut transaction: Option<Transaction> = None;
    for (statement, &location) in statements.iter().zip(locations) {
        match statement {
            Statement::StartTransaction { modes, .. } => {
                transaction.get_or_insert_with(|| {
                    Transaction::new(!modes.contains(&TransactionMode::AccessMode(
                        TransactionAccessMode::ReadOnly,
                    )))
                });
            }
            Statement::Commit { chain, .. }
            | Statement::Rollback {
                chain,
                savepoint: None,
            } => {
                if let Some(active) = transaction.take() {
                    if matches!(statement, Statement::Commit { .. }) {
                        findings.extend(active.pending.into_iter().flatten());
                    }
                    if *chain {
                        transaction = Some(Transaction::new(active.can_commit_writes));
                    }
                }
            }
            Statement::Savepoint { .. }
            | Statement::ReleaseSavepoint { .. }
            | Statement::Rollback {
                savepoint: Some(_), ..
            } => {
                if let Some(active) = &mut transaction {
                    active.pending = None;
                }
            }
            Statement::Set(set) => match access_change(set) {
                Some(AccessChange::Session) => break,
                Some(AccessChange::Transaction) => {
                    if let Some(active) = &mut transaction {
                        active.can_commit_writes = false;
                        active.pending = None;
                    }
                }
                None => {}
            },
            Statement::Explain {
                describe_alias: DescribeAlias::Explain,
                analyze,
                query_plan,
                estimate,
                format,
                options,
                statement,
                ..
            } if source::explain_execution(
                *analyze,
                *query_plan,
                *estimate,
                *format,
                options.as_deref(),
            ) == PostgresSqlExecution::ExecutesForAnalysis =>
            {
                let Some(kind) = direct_write_kind(statement) else {
                    continue;
                };
                let plan = WritePlan {
                    location,
                    kind,
                    autocommit: transaction.is_none(),
                };
                match &mut transaction {
                    Some(active) => {
                        if let Some(pending) = &mut active.pending {
                            pending.push(plan);
                        }
                    }
                    None => findings.push(plan),
                }
            }
            _ => {}
        }
    }
    findings
}

fn direct_write_kind(statement: &Statement) -> Option<&'static str> {
    match statement {
        Statement::Insert(_) => Some("INSERT"),
        Statement::Update(_) => Some("UPDATE"),
        Statement::Delete(_) => Some("DELETE"),
        Statement::Merge(_) => Some("MERGE"),
        Statement::Query(query) => match query.body.as_ref() {
            SetExpr::Insert(_) => Some("INSERT"),
            SetExpr::Update(_) => Some("UPDATE"),
            SetExpr::Delete(_) => Some("DELETE"),
            SetExpr::Merge(_) => Some("MERGE"),
            _ => None,
        },
        _ => None,
    }
}

enum AccessChange {
    Session,
    Transaction,
}

fn access_change(set: &Set) -> Option<AccessChange> {
    match set {
        Set::SetTransaction { modes, session, .. }
            if modes
                .iter()
                .any(|mode| matches!(mode, TransactionMode::AccessMode(_))) =>
        {
            Some(if *session {
                AccessChange::Session
            } else {
                AccessChange::Transaction
            })
        }
        Set::SingleAssignment { variable, .. } => match variable.0.as_slice() {
            [ObjectNamePart::Identifier(name)]
                if name
                    .value
                    .eq_ignore_ascii_case("default_transaction_read_only") =>
            {
                Some(AccessChange::Session)
            }
            [ObjectNamePart::Identifier(name)]
                if name.value.eq_ignore_ascii_case("transaction_read_only") =>
            {
                Some(AccessChange::Transaction)
            }
            _ => None,
        },
        _ => None,
    }
}

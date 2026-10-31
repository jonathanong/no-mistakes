use super::super::{catalog::PreparedCatalog, CallModel};
use crate::codebase::postgres::{
    embedded::helpers::program::{Expr, Stmt},
    sql_identifier_syntax::Span,
};
use crate::fx::FxHashSet;
use std::path::Path;

pub(super) fn accepts(
    catalog: &PreparedCatalog<'_>,
    file: &Path,
    call: &CallModel,
    reference: Span,
    executors: &FxHashSet<usize>,
) -> bool {
    let Some(application) = catalog.application else {
        return false;
    };
    if !matches!(
        application.rule.as_str(),
        "postgres-no-unbounded-sql-identifier" | "postgres-no-transformed-uuid-predicates"
    ) || call.construction
        || !call.members.is_empty()
    {
        return false;
    }
    let Some(profile) = catalog.profile else {
        return false;
    };
    let Some(caller_file) = catalog.files.get(file) else {
        return false;
    };
    let caller = &caller_file.sql_identifier_syntax.recovery;
    if caller.dynamic_scope || call.callee.is_some_and(|id| caller.writes.contains(&id)) {
        return false;
    }
    let callback = call.callee.and_then(|id| caller.callbacks.get(&id));
    let (parameters, typed) = if let Some(parameters) = callback {
        let Some(typed) = caller.executors.get(profile) else {
            return false;
        };
        (parameters, typed)
    } else {
        let Some(key) = call
            .callee
            .and_then(|callee| catalog.typed_binding(file, callee))
        else {
            return false;
        };
        let Some(receiver_file) = catalog.files.get(&key.file) else {
            return false;
        };
        let receiver = &receiver_file.sql_identifier_syntax.recovery;
        if receiver.dynamic_scope {
            return false;
        }
        let Some(parameters) = receiver.signatures.get(&key.callable) else {
            return false;
        };
        let Some(typed) = receiver.executors.get(profile) else {
            return false;
        };
        (parameters, typed)
    };
    call.arguments.iter().enumerate().any(|(index, argument)| {
        let [(id, at)] = argument.references.as_slice() else {
            return false;
        };
        let [Stmt::Return(Some(Expr::Ident(name)), _)] = argument.program.body.as_slice() else {
            return false;
        };
        *at == reference
            && executors.contains(id)
            && name == &format!("binding:{id}")
            && parameters
                .get(index)
                .and_then(|id| *id)
                .is_some_and(|id| typed.contains(&id))
    })
}

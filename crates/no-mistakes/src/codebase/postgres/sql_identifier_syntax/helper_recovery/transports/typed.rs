use super::super::{catalog::PreparedCatalog, ApplicationKey, CallModel, FileModels};
use crate::codebase::postgres::{
    embedded::helpers::program::{Expr, Stmt},
    sql_identifier_syntax::Span,
};
use crate::fx::FxHashSet;
use std::path::Path;

pub(super) fn accepts(
    catalog: &PreparedCatalog<'_>,
    file: &Path,
    application: &ApplicationKey,
    caller: &FileModels,
    call: &CallModel,
    reference: Span,
    executors: &FxHashSet<usize>,
) -> bool {
    if !matches!(
        application.rule.as_str(),
        "postgres-no-unbounded-sql-identifier" | "postgres-no-transformed-uuid-predicates"
    ) || call.construction
        || !call.members.is_empty()
    {
        return false;
    }
    if caller.dynamic_scope || call.callee.is_some_and(|id| caller.writes.contains(&id)) {
        return false;
    }
    let callback = call.callee.and_then(|id| caller.callbacks.get(&id));
    let (parameters, typed) = if let Some(parameters) = callback {
        (parameters.as_slice(), executors)
    } else {
        let Some(receiver) = call
            .callee
            .and_then(|callee| catalog.typed_parameters(file, callee, &application.profile))
        else {
            return false;
        };
        receiver
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

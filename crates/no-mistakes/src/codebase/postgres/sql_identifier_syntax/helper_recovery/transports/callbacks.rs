use super::super::{catalog::PreparedCatalog, ApplicationKey, CallModel, CallableId, FileModels};
use crate::codebase::postgres::embedded::helpers::program::{Expr, Stmt};
use std::path::Path;

pub(super) fn blocks(
    catalog: &PreparedCatalog<'_>,
    file: &Path,
    application: &ApplicationKey,
    caller: &FileModels,
    call: &CallModel,
) -> bool {
    if !matches!(
        application.rule.as_str(),
        "postgres-no-unbounded-sql-identifier" | "postgres-no-transformed-uuid-predicates"
    ) {
        return false;
    }
    let receiving = call.callee.and_then(|id| catalog.typed_signature(file, id));
    call.arguments.iter().enumerate().any(|(index, argument)| {
        let [Stmt::Return(Some(value), mark)] = argument.program.body.as_slice() else {
            return false;
        };
        let expected = receiving.and_then(|(models, parameters)| {
            let callback = models
                .callbacks
                .get(&parameters.get(index).copied().flatten()?)?;
            let executors = models.executors.get(&application.profile)?;
            Some(
                callback
                    .iter()
                    .enumerate()
                    .filter_map(|(index, id)| {
                        id.is_some_and(|id| executors.contains(&id))
                            .then_some(index)
                    })
                    .collect::<Vec<_>>(),
            )
        });
        let actual = match value {
            Expr::Ident(name) => name
                .strip_prefix("binding:")
                .and_then(|id| id.parse().ok())
                .and_then(|id| catalog.typed_signature(file, id)),
            _ => caller
                .signatures
                .get(&CallableId(mark.origin))
                .map(|parameters| (caller, parameters.as_slice())),
        };
        let Some((models, parameters)) = actual else {
            return expected.is_some_and(|indices| !indices.is_empty());
        };
        let required = expected
            .filter(|indices| !indices.is_empty())
            .unwrap_or_else(|| (0..parameters.len()).collect());
        let executors = models.executors.get(&application.profile);
        required.iter().any(|index| {
            !parameters
                .get(*index)
                .copied()
                .flatten()
                .is_some_and(|id| executors.is_some_and(|executors| executors.contains(&id)))
        })
    })
}

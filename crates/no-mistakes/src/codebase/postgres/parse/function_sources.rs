use super::LocatedStatement;
use crate::codebase::postgres::SqlFunctionCallFact;

/// Partition one shared parse into the established AST and auxiliary syntactic
/// call facts; procedural expressions never enter row-bound/lifecycle inputs.
pub(crate) fn partition(
    located: Vec<LocatedStatement>,
) -> (Vec<LocatedStatement>, Vec<SqlFunctionCallFact>) {
    let mut statements = Vec::new();
    let mut functions = Vec::new();
    for mut statement in located {
        functions.append(&mut statement.recovered_functions);
        if statement.function_projection {
            crate::codebase::postgres::function_calls::collect_projected(
                &statement.statement,
                &mut functions,
                statement.synthetic_select,
            );
        } else {
            statements.push(statement);
        }
    }
    (statements, functions)
}

use sqlparser::ast::Statement;

pub(crate) fn walk_executed<'a>(statement: &'a Statement, out: &mut Vec<&'a Statement>) {
    match statement {
        Statement::Explain {
            analyze,
            options,
            statement,
            ..
        } => {
            let option = options
                .iter()
                .flatten()
                .find(|option| option.name.value.eq_ignore_ascii_case("analyze"));
            let enabled = option.is_some_and(|option| {
                option.arg.as_ref().is_none_or(|value| {
                    matches!(
                        value.to_string().to_ascii_lowercase().trim_matches('\''),
                        "true" | "on" | "yes" | "1"
                    )
                })
            });
            if *analyze || enabled {
                walk_executed(statement, out);
            }
        }
        Statement::Prepare { statement, .. } => walk_executed(statement, out),
        Statement::CreateFunction(_) | Statement::CreateProcedure { .. } => {}
        other => out.push(other),
    }
}

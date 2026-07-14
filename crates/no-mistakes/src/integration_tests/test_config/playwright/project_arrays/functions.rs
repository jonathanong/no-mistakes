use super::FnMap;
use oxc_ast::ast::{Declaration, Program, Statement};

pub(super) fn top_level_function_bodies<'a>(program: &'a Program<'a>) -> FnMap<'a> {
    program
        .body
        .iter()
        .filter_map(|statement| {
            let function = match statement {
                Statement::FunctionDeclaration(function) => Some(function),
                Statement::ExportNamedDeclaration(export) => match export.declaration.as_ref() {
                    Some(Declaration::FunctionDeclaration(function)) => Some(function),
                    _ => None,
                },
                _ => None,
            };
            let function = function?;
            Some((
                function.id.as_ref()?.name.to_string(),
                function.body.as_ref()?.as_ref(),
            ))
        })
        .collect()
}

use oxc_ast::ast::{Declaration, ExportDefaultDeclarationKind, Program, Statement};
use std::collections::BTreeMap;

pub(super) fn collect(program: &Program<'_>) -> BTreeMap<String, u32> {
    let reassigned = super::reassigned(program);
    program.body.iter().filter_map(|statement| {
        let function = match statement {
            Statement::FunctionDeclaration(value) => value.as_ref(),
            Statement::ExportDeclaration(value) => match &value.declaration {
                Declaration::FunctionDeclaration(value) => value.as_ref(),
                _ => return None,
            },
            Statement::ExportDefaultDeclaration(value) => match &value.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(value) => value.as_ref(),
                _ => return None,
            },
            _ => return None,
        };
        let name = function.id.as_ref()?.name.as_str();
        // Reuse the existing module-local tag contract with canonical spans.
        // Shadowed or reassigned bindings cannot inherit its prefix evidence.
        ((name.eq_ignore_ascii_case("sql") || name == "String")
            && !reassigned.contains(name)
            && super::super::super::embedded::walk::resolve::functions::looks_like_tag_implementation(function))
            .then(|| (name.to_string(), function.span.start))
    }).collect()
}

use super::super::{import_bindings, objects, ImportBinding};
use crate::integration_tests::test_config::shared_project_arrays;
use oxc_ast::ast::{ObjectExpression, Program, Statement};

pub(in crate::integration_tests::test_config::vitest::project_arrays) fn sourced_reexport(
    program: &Program<'_>,
    exported: &str,
) -> Option<ImportBinding> {
    shared_project_arrays::sourced_reexport(program, exported)
}

pub(in crate::integration_tests::test_config::vitest::project_arrays) fn imported_reexport(
    program: &Program<'_>,
    exported: &str,
) -> Option<ImportBinding> {
    shared_project_arrays::imported_reexport(program, exported, &import_bindings(program), true)
}

pub(in crate::integration_tests::test_config::vitest::project_arrays) fn named_export_object<'a>(
    program: &'a Program<'a>,
    exported: &str,
    bindings: &super::super::ExprMap<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    shared_project_arrays::named_export_object(
        program,
        exported,
        bindings,
        objects::expression_object,
    )
}

pub(in crate::integration_tests::test_config::vitest::project_arrays) fn star_barrel_sources<'a>(
    program: &'a Program<'a>,
) -> impl Iterator<Item = std::borrow::Cow<'a, str>> {
    program.body.iter().filter_map(|s| match s {
        Statement::ExportAllDeclaration(e) if !e.export_kind.is_type() && e.exported.is_none() => {
            Some(crate::ast::js_literal(&e.source))
        }
        _ => None,
    })
}

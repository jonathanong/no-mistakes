use super::super::{import_bindings, objects, ImportBinding};
use crate::integration_tests::test_config::shared_project_arrays;
use oxc_ast::ast::{ObjectExpression, Program};

pub(in crate::integration_tests::test_config::playwright::project_arrays) fn sourced_reexport(
    program: &Program<'_>,
    exported: &str,
) -> Option<ImportBinding> {
    shared_project_arrays::sourced_reexport(program, exported)
}

pub(in crate::integration_tests::test_config::playwright::project_arrays) fn imported_reexport(
    program: &Program<'_>,
    exported: &str,
) -> Option<ImportBinding> {
    shared_project_arrays::imported_reexport(program, exported, &import_bindings(program), false)
}

pub(in crate::integration_tests::test_config::playwright::project_arrays) fn named_export_object<
    'a,
>(
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

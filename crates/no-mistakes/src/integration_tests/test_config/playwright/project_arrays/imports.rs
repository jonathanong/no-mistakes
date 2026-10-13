use crate::integration_tests::test_config::shared_project_arrays::add_es_import_bindings;
pub(super) use crate::integration_tests::test_config::shared_project_arrays::ImportBinding;
use oxc_ast::ast::{Program, Statement};
use std::collections::BTreeMap;

pub(super) fn import_bindings(program: &Program<'_>) -> BTreeMap<String, ImportBinding> {
    let mut bindings = BTreeMap::new();
    for statement in &program.body {
        if let Statement::ImportDeclaration(import) = statement {
            add_es_import_bindings(import, &mut bindings);
        }
    }
    bindings
}

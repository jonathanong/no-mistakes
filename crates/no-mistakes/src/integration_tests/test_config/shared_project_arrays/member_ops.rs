use super::{Context, ExprMap, ImportBinding};
use anyhow::Result;
use oxc_ast::ast::{Expression, ObjectExpression, Program};
use std::collections::BTreeMap;
use std::path::Path;

type ExpressionOptions<T> = fn(&Expression<'_>, &mut Context<'_, '_>) -> Result<Vec<T>>;
type ImportedOptions<T> = fn(&ImportBinding, &mut Context<'_, '_>) -> Result<Vec<T>>;
type ExportedOptions<T> =
    fn(&Program<'_>, &str, &Path, &mut Context<'_, '_>, &str) -> Result<Vec<T>>;
type ExpressionObject =
    for<'a> fn(&'a Expression<'a>, &ExprMap<'a>) -> Option<&'a ObjectExpression<'a>>;

pub(in crate::integration_tests::test_config) struct MemberOps<T> {
    pub expression_options: ExpressionOptions<T>,
    pub imported_options: ImportedOptions<T>,
    pub exported_options: ExportedOptions<T>,
    pub expression_object: ExpressionObject,
    pub import_bindings: fn(&Program<'_>) -> BTreeMap<String, ImportBinding>,
    pub default_identifier_reexport: bool,
}

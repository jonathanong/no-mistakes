use super::{extract_with_argument_offsets, TypeScriptModuleFacts};
use oxc_ast::ast::Program;

pub(super) fn extract_with_arguments(
    program: &Program<'_>,
    collect: bool,
) -> TypeScriptModuleFacts {
    extract_with_argument_offsets(program, collect, None)
}

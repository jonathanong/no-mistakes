use super::callback_argument_index;
use crate::playwright::ast;
use oxc_ast::ast::CallExpression;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HookKind {
    Setup,
    Teardown,
}

pub(crate) fn hook_callback(call: &CallExpression<'_>) -> Option<(usize, HookKind)> {
    let path = ast::expression_path(&call.callee)?;
    if !matches!(path.first().map(String::as_str), Some("test")) {
        return None;
    }
    let kind = if path
        .iter()
        .any(|part| matches!(part.as_str(), "beforeEach" | "beforeAll"))
    {
        HookKind::Setup
    } else if path
        .iter()
        .any(|part| matches!(part.as_str(), "afterEach" | "afterAll"))
    {
        HookKind::Teardown
    } else {
        return None;
    };
    callback_argument_index(call).map(|index| (index, kind))
}

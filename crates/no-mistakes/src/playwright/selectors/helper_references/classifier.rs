use super::super::call_shapes::{callee_is_static_member_named, selector_argument_mode};

pub(super) fn is_helper_reference_call(
    callee: &oxc_ast::ast::Expression<'_>,
    path: &[String],
) -> bool {
    if callee_is_static_member_named(callee, "getByTestId")
        || selector_argument_mode(callee).is_some()
    {
        return false;
    }
    let Some(name) = path.last().map(String::as_str) else {
        return false;
    };
    !is_non_helper_call_name(name) && !is_native_locator_name(name)
}

fn is_native_locator_name(name: &str) -> bool {
    matches!(
        name,
        "getByAltText"
            | "getByLabel"
            | "getByPlaceholder"
            | "getByRole"
            | "getByText"
            | "getByTitle"
    )
}

fn is_non_helper_call_name(name: &str) -> bool {
    matches!(
        name,
        "afterAll"
            | "afterEach"
            | "beforeAll"
            | "beforeEach"
            | "describe"
            | "expect"
            | "fixme"
            | "only"
            | "skip"
            | "slow"
            | "step"
            | "test"
            | "use"
            | "goto"
            | "setTimeout"
            | "toBe"
            | "toBeCloseTo"
            | "toBeDefined"
            | "toBeFalsy"
            | "toBeGreaterThan"
            | "toBeLessThan"
            | "toBeNull"
            | "toBeTruthy"
            | "toBeUndefined"
            | "toBeVisible"
            | "toBeHidden"
            | "toBeEnabled"
            | "toBeDisabled"
            | "toBeChecked"
            | "toBeEditable"
            | "toBeEmpty"
            | "toBeFocused"
            | "toBeInViewport"
            | "toContain"
            | "toEqual"
            | "toHaveBeenCalled"
            | "toHaveBeenCalledWith"
            | "toHaveAttribute"
            | "toHaveClass"
            | "toHaveCount"
            | "toHaveCSS"
            | "toHaveId"
            | "toHaveJSProperty"
            | "toHaveLength"
            | "toHaveProperty"
            | "toHaveText"
            | "toHaveTitle"
            | "toHaveURL"
            | "toHaveValue"
            | "toHaveValues"
            | "toMatch"
            | "toMatchObject"
            | "toMatchSnapshot"
            | "toStrictEqual"
            | "toThrow"
    )
}

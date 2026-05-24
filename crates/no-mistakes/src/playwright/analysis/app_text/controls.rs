use crate::playwright::analysis::types::SelectorRef;
use crate::playwright::selectors::scoped_defaults::ScopedStaticIdentifierDefault;

#[derive(Clone)]
pub(super) struct ControlTextTarget {
    pub(super) role: Option<String>,
    pub(super) hidden: bool,
    pub(super) labelable: bool,
    pub(super) selector_refs: Vec<SelectorRef>,
}

pub(super) struct PendingLabel {
    pub(super) control_ids: Vec<String>,
    pub(super) text: String,
    pub(super) target_control_id: Option<String>,
    pub(super) target_control: Option<ControlTextTarget>,
}

pub(super) fn is_labelable(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    tag: Option<&str>,
    source: &str,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
) -> bool {
    if tag == Some("input")
        && input_type(opening, source, scoped_static_identifier_defaults)
            .eq_ignore_ascii_case("hidden")
    {
        return false;
    }
    matches!(
        tag,
        Some("button" | "input" | "meter" | "output" | "progress" | "select" | "textarea")
    )
}

pub(super) fn input_type_uses_value_text(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    source: &str,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
) -> bool {
    let input_type = input_type(opening, source, scoped_static_identifier_defaults);
    ["button", "reset", "submit"]
        .iter()
        .any(|candidate| input_type.eq_ignore_ascii_case(candidate))
}

pub(super) fn input_type(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    source: &str,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
) -> String {
    super::jsx::string_attr(opening, "type", source, scoped_static_identifier_defaults)
        .unwrap_or_else(|| "text".to_string())
}

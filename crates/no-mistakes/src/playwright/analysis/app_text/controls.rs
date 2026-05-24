use crate::playwright::analysis::types::SelectorRef;
use crate::playwright::selectors::scoped_defaults::ScopedStaticIdentifierDefault;

#[derive(Clone)]
pub(super) struct ControlTextTarget {
    pub(super) role: Option<String>,
    pub(super) selector_refs: Vec<SelectorRef>,
}

pub(super) struct PendingLabel {
    pub(super) control_id: String,
    pub(super) text: String,
    pub(super) target_control_id: Option<String>,
}

pub(super) fn is_labelable(tag: Option<&str>) -> bool {
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
    matches!(
        super::jsx::string_attr(opening, "type", source, scoped_static_identifier_defaults)
            .as_deref()
            .unwrap_or("text"),
        "button" | "reset" | "submit"
    )
}

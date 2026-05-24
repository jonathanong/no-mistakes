use crate::playwright::analysis::types::SelectorRef;

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

pub(super) fn input_type_uses_value_text(opening: &oxc_ast::ast::JSXOpeningElement<'_>) -> bool {
    matches!(
        super::jsx::string_attr(opening, "type")
            .as_deref()
            .unwrap_or("text"),
        "button" | "reset" | "submit"
    )
}

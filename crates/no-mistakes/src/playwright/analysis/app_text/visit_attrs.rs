use super::*;
use crate::playwright::analysis::app_text::controls::input_type;

impl AppTextVisitor<'_> {
    pub(super) fn collect_accessible_attr_targets(
        &mut self,
        opening: &oxc_ast::ast::JSXOpeningElement<'_>,
        tag: Option<&str>,
        role: Option<String>,
        refs: &[SelectorRef],
        visible_name_exists: bool,
    ) {
        let aria_label = self
            .string_attr(opening, "aria-label")
            .and_then(|value| normalize_locator_text(&value));
        let alt = self
            .string_attr(opening, "alt")
            .and_then(|value| normalize_locator_text(&value));
        let title = self
            .string_attr(opening, "title")
            .and_then(|value| normalize_locator_text(&value));
        if let Some(text) = title.as_ref() {
            self.push(AppTextKind::Title, role.clone(), text.clone(), refs);
        }
        if let Some(text) = alt.as_ref().filter(|_| {
            alt_supports_accessible_name(
                opening,
                tag,
                self.source,
                self.scoped_static_identifier_defaults,
            )
        }) {
            self.push(AppTextKind::Alt, role.clone(), (*text).clone(), refs);
        }
        if jsx::attr_exists_at_runtime(opening, "aria-labelledby") {
            return;
        }

        if let Some(text) = aria_label {
            self.push(
                AppTextKind::AccessibleName,
                role.clone(),
                text.clone(),
                refs,
            );
            if is_labelable(
                opening,
                tag,
                self.source,
                self.scoped_static_identifier_defaults,
            ) {
                self.push(AppTextKind::Label, role.clone(), text, refs);
            }
            return;
        }
        if jsx::attr_exists_at_runtime(opening, "aria-label") {
            return;
        }
        if let Some(text) = alt.filter(|_| {
            alt_supports_accessible_name(
                opening,
                tag,
                self.source,
                self.scoped_static_identifier_defaults,
            )
        }) {
            self.push(AppTextKind::AccessibleName, role.clone(), text, refs);
        } else if !visible_name_exists {
            if let Some(text) = title {
                self.push(AppTextKind::AccessibleName, role, text, refs);
            }
        }
    }

    pub(super) fn collect_placeholder_target(
        &mut self,
        opening: &oxc_ast::ast::JSXOpeningElement<'_>,
        tag: Option<&str>,
        role: Option<String>,
        refs: &[SelectorRef],
    ) {
        if !matches!(tag, Some("input" | "textarea")) {
            return;
        }
        if let Some(text) = self
            .string_attr(opening, "placeholder")
            .and_then(|value| normalize_locator_text(&value))
        {
            self.push(AppTextKind::Placeholder, role, text, refs);
        }
    }
}

fn alt_supports_accessible_name(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    tag: Option<&str>,
    source: &str,
    scoped_static_identifier_defaults: &[ScopedStaticIdentifierDefault],
) -> bool {
    matches!(tag, Some("img" | "area"))
        || (tag == Some("input")
            && input_type(opening, source, scoped_static_identifier_defaults)
                .eq_ignore_ascii_case("image"))
}

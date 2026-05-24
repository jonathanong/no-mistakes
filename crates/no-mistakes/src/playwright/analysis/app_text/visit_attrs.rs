use super::*;

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

        if let Some(text) = aria_label {
            self.push(
                AppTextKind::AccessibleName,
                role.clone(),
                text.clone(),
                refs,
            );
            if is_labelable(tag) {
                self.push(AppTextKind::Label, role.clone(), text, refs);
            }
            return;
        }
        if let Some(text) = alt {
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

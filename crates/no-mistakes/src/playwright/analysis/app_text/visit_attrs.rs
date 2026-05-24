use super::*;

impl AppTextVisitor<'_> {
    pub(super) fn collect_accessible_attr_targets(
        &mut self,
        opening: &oxc_ast::ast::JSXOpeningElement<'_>,
        role: Option<String>,
        refs: &[SelectorRef],
    ) {
        for attr in ["aria-label", "title", "alt"] {
            if let Some(text) = self
                .string_attr(opening, attr)
                .and_then(|value| normalize_locator_text(&value))
            {
                self.push(
                    AppTextKind::AccessibleName,
                    role.clone(),
                    text.clone(),
                    refs,
                );
                if attr == "aria-label" {
                    self.push(AppTextKind::Label, role.clone(), text, refs);
                }
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

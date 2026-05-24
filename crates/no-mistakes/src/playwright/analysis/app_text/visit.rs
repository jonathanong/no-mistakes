use super::*;
use oxc_ast_visit::{walk, Visit};

impl<'a> Visit<'a> for AppTextVisitor<'_> {
    fn visit_jsx_element(&mut self, element: &oxc_ast::ast::JSXElement<'a>) {
        let refs = self.element_selector_refs(&element.opening_element);
        let tag = jsx_element_name(&element.opening_element.name);
        let role = self.element_role(&element.opening_element, tag);
        let element_hidden = self.element_is_hidden(&element.opening_element);
        if element_hidden {
            self.hidden_depth += 1;
        }

        self.collect_control_by_id(&element.opening_element, tag, role.clone(), &refs);

        let descendant_texts = descendant_texts(&element.children, self.source);
        if let Some(id) = self.string_attr(&element.opening_element, "id") {
            self.texts_by_id.insert(id, descendant_texts.clone());
        }
        let visible_texts = self.visible_texts(element, role.as_deref(), &descendant_texts);
        let accessible_texts =
            self.accessible_texts(tag, role.as_deref(), &descendant_texts, &visible_texts);
        let visible_name_exists = accessible_texts
            .iter()
            .any(|text| normalize_locator_text(text).is_some());
        let has_aria_label = self
            .string_attr(&element.opening_element, "aria-label")
            .and_then(|value| normalize_locator_text(&value))
            .is_some();
        let has_aria_labelledby = self.has_accessible_name_override(&element.opening_element);
        self.push_normalized_texts(
            AppTextKind::VisibleText,
            role.clone(),
            &visible_texts,
            &refs,
        );
        if !has_aria_label && !has_aria_labelledby {
            self.push_normalized_texts(
                AppTextKind::AccessibleName,
                role.clone(),
                &accessible_texts,
                &refs,
            );
        }
        self.collect_label_targets(element, tag, &accessible_texts);
        self.collect_accessible_attr_targets(
            &element.opening_element,
            tag,
            role.clone(),
            &refs,
            visible_name_exists,
        );
        self.collect_labelledby_targets(&element.opening_element, tag, role.clone(), &refs);
        self.collect_placeholder_target(&element.opening_element, tag, role.clone(), &refs);
        self.collect_input_value_target(&element.opening_element, tag, role, &refs);

        walk::walk_jsx_element(self, element);
        if element_hidden {
            self.hidden_depth -= 1;
        }
    }
}

impl AppTextVisitor<'_> {
    fn has_accessible_name_override(&self, opening: &oxc_ast::ast::JSXOpeningElement<'_>) -> bool {
        if let Some(value) = self.string_attr(opening, "aria-labelledby") {
            return normalize_locator_text(&value).is_some();
        }
        jsx::attr_exists_at_runtime(opening, "aria-labelledby")
    }
}

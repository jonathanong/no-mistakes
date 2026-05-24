use super::*;
use crate::playwright::analysis::app_text::controls::{input_type_uses_value_text, is_labelable};
use oxc_ast_visit::{walk, Visit};

impl<'a> Visit<'a> for AppTextVisitor<'_> {
    fn visit_jsx_element(&mut self, element: &oxc_ast::ast::JSXElement<'a>) {
        let refs = selector_refs(
            &element.opening_element,
            self.source,
            self.settings,
            self.scoped_static_identifier_defaults,
        );
        let tag = jsx_element_name(&element.opening_element.name);
        let role = element_role(
            &element.opening_element,
            tag,
            self.source,
            self.scoped_static_identifier_defaults,
        );
        let element_hidden = self.element_is_hidden(&element.opening_element);
        if element_hidden {
            self.hidden_depth += 1;
        }

        if is_labelable(tag) {
            if let Some(id) = self.string_attr(&element.opening_element, "id") {
                self.controls_by_id.insert(
                    id,
                    ControlTextTarget {
                        role: role.clone(),
                        hidden: self.hidden_depth > 0,
                        selector_refs: refs.clone(),
                    },
                );
            }
        }

        let descendant_texts = descendant_texts(&element.children, self.source);
        if let Some(id) = self.string_attr(&element.opening_element, "id") {
            self.texts_by_id.insert(id, descendant_texts.clone());
        }
        let visible_texts = direct_child_texts(&element.children, self.source);
        let accessible_texts = if role.is_some() || tag == Some("label") {
            descendant_texts
        } else {
            visible_texts.clone()
        };
        for text in &visible_texts {
            if let Some(text) = normalize_locator_text(text) {
                self.push(AppTextKind::VisibleText, role.clone(), text, &refs);
            }
        }
        for text in &accessible_texts {
            if let Some(text) = normalize_locator_text(text) {
                self.push(AppTextKind::AccessibleName, role.clone(), text, &refs);
            }
        }
        if tag == Some("label") {
            let label_for = self
                .string_attr(&element.opening_element, "for")
                .or_else(|| self.string_attr(&element.opening_element, "htmlFor"));
            if let Some(control_id) = label_for {
                for text in accessible_texts {
                    if let Some(text) = normalize_locator_text(&text) {
                        self.pending_labels.push(PendingLabel {
                            control_id: control_id.clone(),
                            text,
                            target_control_id: None,
                        });
                    }
                }
            } else if let Some(control) =
                self.nested_label_control(&element.children, self.hidden_depth > 0)
            {
                for text in accessible_texts {
                    if let Some(text) = normalize_locator_text(&text) {
                        self.push_control_name_targets(&control, text);
                    }
                }
            } else {
                for text in accessible_texts {
                    if let Some(text) = normalize_locator_text(&text) {
                        self.push(AppTextKind::Label, role.clone(), text, &refs);
                    }
                }
            }
        }

        for attr in ["aria-label", "title", "alt"] {
            if let Some(text) = self
                .string_attr(&element.opening_element, attr)
                .and_then(|value| normalize_locator_text(&value))
            {
                self.push(
                    AppTextKind::AccessibleName,
                    role.clone(),
                    text.clone(),
                    &refs,
                );
                if attr == "aria-label" {
                    self.push(AppTextKind::Label, role.clone(), text, &refs);
                }
            }
        }

        if let Some(labelledby) = self.string_attr(&element.opening_element, "aria-labelledby") {
            for control_id in labelledby.split_whitespace() {
                self.pending_labels.push(PendingLabel {
                    control_id: control_id.to_string(),
                    text: String::new(),
                    target_control_id: self.string_attr(&element.opening_element, "id"),
                });
            }
        }

        if let Some(text) = self
            .string_attr(&element.opening_element, "placeholder")
            .and_then(|value| normalize_locator_text(&value))
        {
            self.push(AppTextKind::Placeholder, role.clone(), text, &refs);
        }

        if tag == Some("input")
            && input_type_uses_value_text(
                &element.opening_element,
                self.source,
                self.scoped_static_identifier_defaults,
            )
        {
            if let Some(text) = self
                .string_attr(&element.opening_element, "value")
                .and_then(|value| normalize_locator_text(&value))
            {
                self.push(AppTextKind::VisibleText, role, text, &refs);
            }
        }

        walk::walk_jsx_element(self, element);
        if element_hidden {
            self.hidden_depth -= 1;
        }
    }
}

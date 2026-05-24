use super::*;
use crate::playwright::analysis::app_text::controls::{input_type_uses_value_text, is_labelable};

impl AppTextVisitor<'_> {
    pub(super) fn element_selector_refs(
        &self,
        opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    ) -> Vec<SelectorRef> {
        selector_refs(
            opening,
            self.source,
            self.settings,
            self.scoped_static_identifier_defaults,
        )
    }

    pub(super) fn element_role(
        &self,
        opening: &oxc_ast::ast::JSXOpeningElement<'_>,
        tag: Option<&str>,
    ) -> Option<String> {
        element_role(
            opening,
            tag,
            self.source,
            self.scoped_static_identifier_defaults,
        )
    }

    pub(super) fn collect_control_by_id(
        &mut self,
        opening: &oxc_ast::ast::JSXOpeningElement<'_>,
        tag: Option<&str>,
        role: Option<String>,
        refs: &[SelectorRef],
    ) {
        let labelable = is_labelable(tag);
        if !labelable && role.is_none() {
            return;
        }
        if let Some(id) = self.string_attr(opening, "id") {
            self.controls_by_id.insert(
                id,
                ControlTextTarget {
                    role,
                    hidden: self.hidden_depth > 0,
                    labelable,
                    selector_refs: refs.to_vec(),
                },
            );
        }
    }

    pub(super) fn visible_texts(
        &self,
        element: &oxc_ast::ast::JSXElement<'_>,
        role: Option<&str>,
        descendant_texts: &[String],
    ) -> Vec<String> {
        if role.is_some() {
            descendant_texts.to_vec()
        } else {
            direct_child_texts(&element.children, self.source)
        }
    }

    pub(super) fn accessible_texts(
        &self,
        tag: Option<&str>,
        role: Option<&str>,
        descendant_texts: &[String],
        visible_texts: &[String],
    ) -> Vec<String> {
        if role.is_some() || tag == Some("label") {
            descendant_texts.to_vec()
        } else {
            visible_texts.to_vec()
        }
    }

    pub(super) fn push_normalized_texts(
        &mut self,
        kind: AppTextKind,
        role: Option<String>,
        texts: &[String],
        refs: &[SelectorRef],
    ) {
        for text in texts {
            if let Some(text) = normalize_locator_text(text) {
                self.push(kind.clone(), role.clone(), text, refs);
            }
        }
    }

    pub(super) fn collect_label_targets(
        &mut self,
        element: &oxc_ast::ast::JSXElement<'_>,
        tag: Option<&str>,
        accessible_texts: &[String],
    ) {
        if tag != Some("label") {
            return;
        }
        let opening = &element.opening_element;
        if let Some(control_id) = self
            .string_attr(opening, "for")
            .or_else(|| self.string_attr(opening, "htmlFor"))
        {
            self.push_pending_label_targets(control_id, accessible_texts);
        } else if let Some(control) =
            self.nested_label_control(&element.children, self.hidden_depth > 0)
        {
            self.push_wrapped_label_targets(&control, accessible_texts);
        }
    }

    fn push_pending_label_targets(&mut self, control_id: String, texts: &[String]) {
        for text in texts {
            if let Some(text) = normalize_locator_text(text) {
                self.pending_labels.push(PendingLabel {
                    control_ids: vec![control_id.clone()],
                    text,
                    target_control_id: None,
                    target_control: None,
                });
            }
        }
    }

    fn push_wrapped_label_targets(&mut self, control: &ControlTextTarget, texts: &[String]) {
        for text in texts {
            if let Some(text) = normalize_locator_text(text) {
                self.push_control_name_targets(control, text);
            }
        }
    }

    pub(super) fn collect_labelledby_targets(
        &mut self,
        opening: &oxc_ast::ast::JSXOpeningElement<'_>,
        tag: Option<&str>,
        role: Option<String>,
        refs: &[SelectorRef],
    ) {
        let Some(labelledby) = self.string_attr(opening, "aria-labelledby") else {
            return;
        };
        let target_control_id = self.string_attr(opening, "id");
        let target_control = target_control_id.is_none().then(|| ControlTextTarget {
            role,
            hidden: self.hidden_depth > 0,
            labelable: is_labelable(tag),
            selector_refs: refs.to_vec(),
        });
        self.pending_labels.push(PendingLabel {
            control_ids: labelledby.split_whitespace().map(str::to_string).collect(),
            text: String::new(),
            target_control_id,
            target_control,
        });
    }

    pub(super) fn collect_input_value_target(
        &mut self,
        opening: &oxc_ast::ast::JSXOpeningElement<'_>,
        tag: Option<&str>,
        role: Option<String>,
        refs: &[SelectorRef],
    ) {
        if tag != Some("input")
            || !input_type_uses_value_text(
                opening,
                self.source,
                self.scoped_static_identifier_defaults,
            )
        {
            return;
        }
        if let Some(text) = self
            .string_attr(opening, "value")
            .and_then(|value| normalize_locator_text(&value))
        {
            self.push(AppTextKind::VisibleText, role, text, refs);
        }
    }
}

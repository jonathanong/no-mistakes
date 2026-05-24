use super::*;

impl AppTextVisitor<'_> {
    fn resolve_pending_label(
        &self,
        label: PendingLabel,
    ) -> Option<(ControlTextTarget, Vec<String>)> {
        if let Some(target_control) = label.target_control {
            return labelledby_texts(&label.control_ids, &self.texts_by_id)
                .map(|texts| (target_control, texts));
        }
        if let Some(target_control_id) = &label.target_control_id {
            let control = self.controls_by_id.get(target_control_id).cloned()?;
            return labelledby_texts(&label.control_ids, &self.texts_by_id)
                .map(|texts| (control, texts));
        }
        let control_id = label.control_ids.first()?;
        let control = self.controls_by_id.get(control_id).cloned()?;
        Some((control, vec![label.text]))
    }

    pub(super) fn finish(&mut self) {
        for label in std::mem::take(&mut self.pending_labels) {
            let Some((control, texts)) = self.resolve_pending_label(label) else {
                continue;
            };
            for text in texts {
                if let Some(text) = normalize_locator_text(&text) {
                    self.push_control_name_targets(&control, text);
                }
            }
        }
    }

    pub(super) fn push_control_name_targets(&mut self, control: &ControlTextTarget, text: String) {
        if control.labelable {
            self.targets.push(AppTextTarget {
                file: self.path.to_path_buf(),
                app_file: Arc::new(relative_string(self.root, self.path)),
                kind: AppTextKind::Label,
                role: control.role.clone(),
                text: text.clone(),
                hidden: control.hidden,
                selector_refs: control.selector_refs.clone(),
            });
        }
        self.targets.push(AppTextTarget {
            file: self.path.to_path_buf(),
            app_file: Arc::new(relative_string(self.root, self.path)),
            kind: AppTextKind::AccessibleName,
            role: control.role.clone(),
            text,
            hidden: control.hidden,
            selector_refs: control.selector_refs.clone(),
        });
    }
}

fn labelledby_texts(
    control_ids: &[String],
    texts_by_id: &HashMap<String, Vec<String>>,
) -> Option<Vec<String>> {
    let texts = control_ids
        .iter()
        .filter_map(|control_id| texts_by_id.get(control_id))
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    (!texts.is_empty()).then(|| vec![texts.join(" ")])
}

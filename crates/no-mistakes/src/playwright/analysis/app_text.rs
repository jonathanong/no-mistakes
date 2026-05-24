use crate::playwright::analysis::app_collect::collect_selector_source_files;
use crate::playwright::analysis::text_types::{normalize_locator_text, AppTextKind, AppTextTarget};
use crate::playwright::analysis::types::SelectorRef;
use crate::playwright::config::Settings;
use crate::playwright::fsutil::{build_globset, relative_string};
use anyhow::{Context, Result};
use controls::{input_type_uses_value_text, is_labelable, ControlTextTarget, PendingLabel};
use jsx::*;
use oxc_ast_visit::{walk, Visit};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

mod controls;
mod extract;
mod jsx;
use extract::extract_app_text_targets;

pub(crate) fn collect_app_text_targets(
    root: &Path,
    settings: &Settings,
) -> Result<Vec<AppTextTarget>> {
    let include = build_globset(&settings.selector_include)?;
    let exclude = build_globset(&settings.selector_exclude)?;
    let include_all = settings.selector_include.is_empty();
    let source_files =
        collect_selector_source_files(root, settings, &include, &exclude, include_all);

    let mut targets = source_files
        .par_iter()
        .try_fold(Vec::new, |mut targets, path| -> Result<_> {
            let source = std::fs::read_to_string(path)
                .context(format!("reading selector source {}", path.display()))?;
            targets.extend(extract_app_text_targets(root, path, &source, settings)?);
            Ok(targets)
        })
        .try_reduce(Vec::new, |mut left, mut right| -> Result<_> {
            left.append(&mut right);
            Ok(left)
        })?;
    targets.sort();
    targets.dedup();
    Ok(targets)
}

struct AppTextVisitor<'a> {
    root: &'a Path,
    path: &'a Path,
    settings: &'a Settings,
    targets: Vec<AppTextTarget>,
    controls_by_id: HashMap<String, ControlTextTarget>,
    pending_labels: Vec<PendingLabel>,
    texts_by_id: HashMap<String, Vec<String>>,
}

impl<'a> Visit<'a> for AppTextVisitor<'_> {
    fn visit_jsx_element(&mut self, element: &oxc_ast::ast::JSXElement<'a>) {
        let refs = selector_refs(&element.opening_element, self.settings);
        let tag = jsx_element_name(&element.opening_element.name);
        let role = element_role(&element.opening_element, tag);

        if is_labelable(tag) {
            if let Some(id) = string_attr(&element.opening_element, "id") {
                self.controls_by_id.insert(
                    id,
                    ControlTextTarget {
                        role: role.clone(),
                        selector_refs: refs.clone(),
                    },
                );
            }
        }

        let descendant_texts = descendant_texts(&element.children);
        if let Some(id) = string_attr(&element.opening_element, "id") {
            self.texts_by_id.insert(id, descendant_texts.clone());
        }
        let visible_texts = direct_child_texts(&element.children);
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
            let label_for = string_attr(&element.opening_element, "for")
                .or_else(|| string_attr(&element.opening_element, "htmlFor"));
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
            } else {
                for text in accessible_texts {
                    if let Some(text) = normalize_locator_text(&text) {
                        self.push(AppTextKind::Label, role.clone(), text, &refs);
                    }
                }
            }
        }

        for attr in ["aria-label", "title", "alt"] {
            if let Some(text) = string_attr(&element.opening_element, attr)
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

        if let Some(labelledby) = string_attr(&element.opening_element, "aria-labelledby") {
            for control_id in labelledby.split_whitespace() {
                self.pending_labels.push(PendingLabel {
                    control_id: control_id.to_string(),
                    text: String::new(),
                    target_control_id: string_attr(&element.opening_element, "id"),
                });
            }
        }

        if let Some(text) = string_attr(&element.opening_element, "placeholder")
            .and_then(|value| normalize_locator_text(&value))
        {
            self.push(AppTextKind::Placeholder, role.clone(), text, &refs);
        }

        if tag == Some("input") && input_type_uses_value_text(&element.opening_element) {
            if let Some(text) = string_attr(&element.opening_element, "value")
                .and_then(|value| normalize_locator_text(&value))
            {
                self.push(AppTextKind::VisibleText, role, text, &refs);
            }
        }

        walk::walk_jsx_element(self, element);
    }
}

impl AppTextVisitor<'_> {
    fn push(
        &mut self,
        kind: AppTextKind,
        role: Option<String>,
        text: String,
        selector_refs: &[SelectorRef],
    ) {
        self.targets.push(AppTextTarget {
            file: self.path.to_path_buf(),
            app_file: Arc::new(relative_string(self.root, self.path)),
            kind,
            role,
            text,
            selector_refs: selector_refs.to_vec(),
        });
    }

    fn finish(&mut self) {
        for label in std::mem::take(&mut self.pending_labels) {
            let (control_id, texts) = if let Some(target_control_id) = &label.target_control_id {
                let Some(texts) = self.texts_by_id.get(&label.control_id) else {
                    continue;
                };
                (target_control_id, texts.clone())
            } else {
                (&label.control_id, vec![label.text])
            };
            let Some(control) = self.controls_by_id.get(control_id).cloned() else {
                continue;
            };
            for text in texts {
                if let Some(text) = normalize_locator_text(&text) {
                    self.push_control_name_targets(&control, text);
                }
            }
        }
    }

    fn push_control_name_targets(&mut self, control: &ControlTextTarget, text: String) {
        for kind in [AppTextKind::Label, AppTextKind::AccessibleName] {
            self.targets.push(AppTextTarget {
                file: self.path.to_path_buf(),
                app_file: Arc::new(relative_string(self.root, self.path)),
                kind,
                role: control.role.clone(),
                text: text.clone(),
                selector_refs: control.selector_refs.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests;

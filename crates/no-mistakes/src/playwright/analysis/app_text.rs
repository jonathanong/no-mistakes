use crate::playwright::analysis::app_collect::collect_selector_source_files;
use crate::playwright::analysis::text_types::{normalize_locator_text, AppTextKind, AppTextTarget};
use crate::playwright::analysis::types::SelectorRef;
use crate::playwright::config::Settings;
use crate::playwright::fsutil::{build_globset, relative_string};
use crate::playwright::selectors::scoped_defaults::ScopedStaticIdentifierDefault;
use anyhow::{Context, Result};
use controls::{ControlTextTarget, PendingLabel};
use jsx::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

mod controls;
mod elements;
mod extract;
mod jsx;
mod jsx_text;
mod roles;
mod visit;
use elements::*;
use extract::extract_app_text_targets;
use jsx_text::*;
use roles::*;

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
    source: &'a str,
    settings: &'a Settings,
    scoped_static_identifier_defaults: &'a [ScopedStaticIdentifierDefault],
    targets: Vec<AppTextTarget>,
    controls_by_id: HashMap<String, ControlTextTarget>,
    pending_labels: Vec<PendingLabel>,
    texts_by_id: HashMap<String, Vec<String>>,
    hidden_depth: usize,
}

impl AppTextVisitor<'_> {
    fn string_attr(
        &self,
        opening: &oxc_ast::ast::JSXOpeningElement<'_>,
        name: &str,
    ) -> Option<String> {
        string_attr(
            opening,
            name,
            self.source,
            self.scoped_static_identifier_defaults,
        )
    }

    fn element_is_hidden(&self, opening: &oxc_ast::ast::JSXOpeningElement<'_>) -> bool {
        bool_attr(opening, "hidden").unwrap_or(false)
            || self
                .string_attr(opening, "aria-hidden")
                .is_some_and(|value| value == "true")
    }

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
            hidden: self.hidden_depth > 0,
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
                hidden: control.hidden,
                selector_refs: control.selector_refs.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests;

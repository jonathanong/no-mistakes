use crate::playwright::analysis::app_collect::collect_selector_source_files;
use crate::playwright::analysis::text_types::{normalize_locator_text, AppTextKind, AppTextTarget};
use crate::playwright::analysis::types::SelectorRef;
use crate::playwright::config::Settings;
use crate::playwright::fsutil::{build_globset, relative_string};
use crate::playwright::selectors::scoped_defaults::ScopedStaticIdentifierDefault;
use anyhow::{Context, Result};
use controls::{is_labelable, ControlTextTarget, PendingLabel};
use jsx::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

mod controls;
mod elements;
mod extract;
mod finish;
mod jsx;
mod jsx_text;
mod roles;
mod visit;
mod visit_attrs;
mod visit_helpers;
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
            || aria_bool_attr(opening, "aria-hidden").unwrap_or(false)
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

    fn nested_label_control(
        &self,
        children: &[oxc_ast::ast::JSXChild<'_>],
        inherited_hidden: bool,
    ) -> Option<ControlTextTarget> {
        for child in children {
            match child {
                oxc_ast::ast::JSXChild::Element(element) => {
                    let tag = jsx_element_name(&element.opening_element.name);
                    let hidden =
                        inherited_hidden || self.element_is_hidden(&element.opening_element);
                    if is_labelable(
                        &element.opening_element,
                        tag,
                        self.source,
                        self.scoped_static_identifier_defaults,
                    ) {
                        return Some(ControlTextTarget {
                            role: element_role(
                                &element.opening_element,
                                tag,
                                self.source,
                                self.scoped_static_identifier_defaults,
                            ),
                            hidden,
                            labelable: true,
                            selector_refs: selector_refs(
                                &element.opening_element,
                                self.source,
                                self.settings,
                                self.scoped_static_identifier_defaults,
                            ),
                        });
                    }
                    if let Some(control) = self.nested_label_control(&element.children, hidden) {
                        return Some(control);
                    }
                }
                oxc_ast::ast::JSXChild::Fragment(fragment) => {
                    if let Some(control) =
                        self.nested_label_control(&fragment.children, inherited_hidden)
                    {
                        return Some(control);
                    }
                }
                _ => {}
            }
        }
        None
    }
}

#[cfg(test)]
mod tests;

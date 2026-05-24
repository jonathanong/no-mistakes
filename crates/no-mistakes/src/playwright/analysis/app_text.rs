use crate::playwright::analysis::app_collect::collect_selector_source_files;
use crate::playwright::analysis::text_types::{normalize_locator_text, AppTextKind, AppTextTarget};
use crate::playwright::analysis::types::SelectorRef;
use crate::playwright::ast;
use crate::playwright::config::Settings;
use crate::playwright::fsutil::{build_globset, relative_string};
use anyhow::{Context, Result};
use jsx::{direct_child_texts, element_role, jsx_element_name, selector_refs, string_attr};
use oxc_ast_visit::{walk, Visit};
use rayon::prelude::*;
use std::path::Path;
use std::sync::Arc;

mod jsx;

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

fn extract_app_text_targets(
    root: &Path,
    path: &Path,
    source: &str,
    settings: &Settings,
) -> Result<Vec<AppTextTarget>> {
    ast::with_program(path, source, |program, _| {
        let mut visitor = AppTextVisitor {
            root,
            path,
            settings,
            targets: Vec::new(),
        };
        visitor.visit_program(program);
        visitor.targets
    })
}

struct AppTextVisitor<'a> {
    root: &'a Path,
    path: &'a Path,
    settings: &'a Settings,
    targets: Vec<AppTextTarget>,
}

impl<'a> Visit<'a> for AppTextVisitor<'_> {
    fn visit_jsx_element(&mut self, element: &oxc_ast::ast::JSXElement<'a>) {
        let refs = selector_refs(&element.opening_element, self.settings);
        let tag = jsx_element_name(&element.opening_element.name);
        let role = element_role(&element.opening_element, tag);

        for text in direct_child_texts(&element.children) {
            if let Some(text) = normalize_locator_text(&text) {
                self.push(AppTextKind::VisibleText, role.clone(), text.clone(), &refs);
                if tag == Some("label") {
                    self.push(AppTextKind::Label, role.clone(), text.clone(), &refs);
                }
                self.push(AppTextKind::AccessibleName, role.clone(), text, &refs);
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

        if let Some(text) = string_attr(&element.opening_element, "placeholder")
            .and_then(|value| normalize_locator_text(&value))
        {
            self.push(AppTextKind::Placeholder, role, text, &refs);
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
}

#[cfg(test)]
mod tests;

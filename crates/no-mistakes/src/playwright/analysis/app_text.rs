use crate::playwright::analysis::app_collect::collect_selector_source_files;
use crate::playwright::analysis::text_types::{normalize_locator_text, AppTextKind, AppTextTarget};
use crate::playwright::analysis::types::SelectorRef;
use crate::playwright::ast;
use crate::playwright::config::Settings;
use crate::playwright::fsutil::{build_globset, relative_string};
use anyhow::{Context, Result};
use oxc_ast_visit::{walk, Visit};
use rayon::prelude::*;
use std::path::Path;
use std::sync::Arc;

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

        for text in direct_child_texts(&element.children) {
            if let Some(text) = normalize_locator_text(&text) {
                self.push(AppTextKind::VisibleText, text.clone(), &refs);
                if tag == Some("label") {
                    self.push(AppTextKind::Label, text.clone(), &refs);
                }
                self.push(AppTextKind::AccessibleName, text, &refs);
            }
        }

        for attr in ["aria-label", "title", "alt"] {
            if let Some(text) = string_attr(&element.opening_element, attr)
                .and_then(|value| normalize_locator_text(&value))
            {
                self.push(AppTextKind::AccessibleName, text.clone(), &refs);
                if attr == "aria-label" {
                    self.push(AppTextKind::Label, text, &refs);
                }
            }
        }

        if let Some(text) = string_attr(&element.opening_element, "placeholder")
            .and_then(|value| normalize_locator_text(&value))
        {
            self.push(AppTextKind::Placeholder, text, &refs);
        }

        walk::walk_jsx_element(self, element);
    }
}

impl AppTextVisitor<'_> {
    fn push(&mut self, kind: AppTextKind, text: String, selector_refs: &[SelectorRef]) {
        self.targets.push(AppTextTarget {
            file: self.path.to_path_buf(),
            app_file: Arc::new(relative_string(self.root, self.path)),
            kind,
            text,
            selector_refs: selector_refs.to_vec(),
        });
    }
}

fn direct_child_texts(children: &[oxc_ast::ast::JSXChild<'_>]) -> Vec<String> {
    children
        .iter()
        .filter_map(|child| match child {
            oxc_ast::ast::JSXChild::Text(text) => Some(text.value.to_string()),
            oxc_ast::ast::JSXChild::ExpressionContainer(container) => match &container.expression {
                oxc_ast::ast::JSXExpression::StringLiteral(literal) => {
                    Some(literal.value.to_string())
                }
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn selector_refs(
    opening: &oxc_ast::ast::JSXOpeningElement<'_>,
    settings: &Settings,
) -> Vec<SelectorRef> {
    let component = jsx_element_name(&opening.name)
        .and_then(|name| name.chars().next())
        .is_some_and(|ch| !ch.is_ascii_lowercase());
    let mut refs = Vec::new();
    for item in &opening.attributes {
        let oxc_ast::ast::JSXAttributeItem::Attribute(attribute) = item else {
            continue;
        };
        let Some(name) = jsx_attribute_name(&attribute.name) else {
            continue;
        };
        let mapped = if settings.selector_attributes.iter().any(|attr| attr == name) {
            Some(name)
        } else if component {
            settings
                .component_selector_attributes
                .get(name)
                .map(String::as_str)
        } else {
            None
        };
        let Some(attribute_name) = mapped else {
            continue;
        };
        let Some(value) = jsx_attr_string(attribute.value.as_ref()) else {
            continue;
        };
        refs.push(SelectorRef {
            attribute: attribute_name.to_string(),
            value,
        });
    }
    refs.sort();
    refs.dedup();
    refs
}

fn string_attr(opening: &oxc_ast::ast::JSXOpeningElement<'_>, name: &str) -> Option<String> {
    for item in &opening.attributes {
        let oxc_ast::ast::JSXAttributeItem::Attribute(attribute) = item else {
            continue;
        };
        if jsx_attribute_name(&attribute.name) == Some(name) {
            return jsx_attr_string(attribute.value.as_ref());
        }
    }
    None
}

fn jsx_attr_string(value: Option<&oxc_ast::ast::JSXAttributeValue<'_>>) -> Option<String> {
    match value? {
        oxc_ast::ast::JSXAttributeValue::StringLiteral(literal) => Some(literal.value.to_string()),
        oxc_ast::ast::JSXAttributeValue::ExpressionContainer(container) => {
            match &container.expression {
                oxc_ast::ast::JSXExpression::StringLiteral(literal) => {
                    Some(literal.value.to_string())
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn jsx_attribute_name<'a>(name: &'a oxc_ast::ast::JSXAttributeName<'a>) -> Option<&'a str> {
    match name {
        oxc_ast::ast::JSXAttributeName::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn jsx_element_name<'a>(name: &'a oxc_ast::ast::JSXElementName<'a>) -> Option<&'a str> {
    match name {
        oxc_ast::ast::JSXElementName::Identifier(identifier) => Some(identifier.name.as_str()),
        oxc_ast::ast::JSXElementName::IdentifierReference(identifier) => {
            Some(identifier.name.as_str())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests;

use super::types::Options;
use crate::codebase::ts_resolver::{load_tsconfig, normalize_path, TsConfig};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::Result;
use oxc_allocator::Allocator;
use oxc_ast::ast::{ArrayExpressionElement, Expression, ObjectExpression, ObjectProperty};
use oxc_ast_visit::{walk, Visit};
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::path::{Path, PathBuf};

pub(super) fn resolve_tsconfig(root: &Path, tsconfig_path: Option<&Path>) -> Result<TsConfig> {
    match tsconfig_path {
        Some(path) => load_tsconfig(path),
        None => match crate::codebase::ts_resolver::find_tsconfig(root) {
            Some(path) => load_tsconfig(&path),
            None => Ok(TsConfig {
                dir: root.to_path_buf(),
                paths: vec![],
                paths_dir: root.to_path_buf(),
                base_url: None,
            }),
        },
    }
}

pub(super) fn effective_story_patterns(
    root: &Path,
    project_root: &Path,
    config: &NoMistakesConfig,
    opts: &Options,
) -> Vec<String> {
    if !opts.stories.is_empty() {
        return opts.stories.clone();
    }
    let mut patterns = Vec::new();
    if let Some(configs) = config.tests.storybook.configs.as_ref() {
        for config_path in configs.values() {
            let config_path = resolve_storybook_config_path(root, project_root, &config_path);
            let Ok(source) = std::fs::read_to_string(&config_path) else {
                continue;
            };
            let base = config_path.parent().unwrap_or(project_root);
            for story in extract_storybook_story_patterns(&source) {
                patterns.push(project_relative_pattern(project_root, base, &story));
            }
        }
    }
    if patterns.is_empty() {
        patterns.push("**/*.stories.{ts,tsx,js,jsx}".to_string());
    }
    patterns.sort();
    patterns.dedup();
    patterns
}

fn resolve_storybook_config_path(root: &Path, project_root: &Path, config_path: &str) -> PathBuf {
    let path = Path::new(config_path);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    let from_root = root.join(path);
    if from_root.exists() {
        from_root
    } else {
        project_root.join(path)
    }
}

fn extract_storybook_story_patterns(source: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        return Vec::new();
    }
    let mut visitor = StorybookConfigVisitor {
        source,
        patterns: Vec::new(),
    };
    visitor.visit_program(&parsed.program);
    visitor.patterns
}

pub(super) fn project_relative_pattern(project_root: &Path, base: &Path, pattern: &str) -> String {
    let project_root = normalize_path(project_root);
    let pattern_path = Path::new(pattern);
    if pattern_path.is_absolute() {
        return relative_slash_path(&project_root, &normalize_path(pattern_path));
    }
    let joined = base.join(pattern_path);
    relative_slash_path(&project_root, &normalize_path(&joined))
}

struct StorybookConfigVisitor<'a> {
    source: &'a str,
    patterns: Vec<String>,
}

impl<'a> Visit<'a> for StorybookConfigVisitor<'a> {
    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        if crate::codebase::ts_source::static_property_key_name(&property.key) == Some("stories") {
            self.patterns
                .extend(story_patterns_from_expression(&property.value, self.source));
        }
        walk::walk_object_property(self, property);
    }
}

fn story_patterns_from_expression(expression: &Expression<'_>, source: &str) -> Vec<String> {
    let Expression::ArrayExpression(array) = parenthesized_expression(expression) else {
        return Vec::new();
    };
    array
        .elements
        .iter()
        .filter_map(|element| story_pattern_from_element(element, source))
        .collect()
}

fn story_pattern_from_element(
    element: &ArrayExpressionElement<'_>,
    source: &str,
) -> Option<String> {
    match element {
        ArrayExpressionElement::StringLiteral(literal) => Some(literal.value.to_string()),
        ArrayExpressionElement::TemplateLiteral(template) if template.expressions.is_empty() => {
            Some(crate::ast::template_literal_text(template, source))
        }
        ArrayExpressionElement::ObjectExpression(object) => {
            story_pattern_from_object(object, source)
        }
        _ => None,
    }
}

fn story_pattern_from_object(object: &ObjectExpression<'_>, source: &str) -> Option<String> {
    let directory = object_string_property(object, "directory", source)?;
    let files = object_string_property(object, "files", source)
        .unwrap_or_else(|| "**/*.stories.@(js|jsx|mjs|ts|tsx)".to_string());
    Some(format!("{}/{}", directory.trim_end_matches('/'), files))
}

fn object_string_property(
    object: &ObjectExpression<'_>,
    name: &str,
    source: &str,
) -> Option<String> {
    object.properties.iter().find_map(|property| {
        let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property.method {
            return None;
        }
        let key = crate::codebase::ts_source::static_property_key_name(&property.key)?;
        (key == name).then(|| optional_string(&property.value, source))?
    })
}

fn optional_string(expression: &Expression<'_>, source: &str) -> Option<String> {
    match parenthesized_expression(expression) {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            Some(crate::ast::template_literal_text(template, source))
        }
        _ => None,
    }
}

fn parenthesized_expression<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(parenthesized) => {
            parenthesized_expression(&parenthesized.expression)
        }
        _ => expression,
    }
}

//! Static `optimizeDeps.include` entries from a Vite config module.

use super::config_object::default_object;
use super::specifier::{coverage_name, nested_pairs, package_name};
use crate::ast::{js_literal, property_key_name, template_quasi_text};
use crate::codebase::ts_source::byte_offset_to_line;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{ArrayExpressionElement, Expression, ObjectExpression, ObjectPropertyKind};
use oxc_span::GetSpan;
use std::path::Path;

pub(super) struct NestedInclude {
    pub parent: String,
    pub child: String,
    pub line: usize,
    pub text: String,
}

pub(super) enum ParsedConfig {
    Includes(Vec<NestedInclude>),
    Unanalyzable { line: usize, reason: String },
}

pub(super) fn parse_vite_config(
    path: &Path,
    source: &str,
    allow_imported_include: bool,
) -> ParsedConfig {
    match crate::ast::with_program(path, source, |program, source| {
        extract(program, source, allow_imported_include)
    }) {
        Ok(parsed) => parsed,
        Err(_) => ParsedConfig::Unanalyzable {
            line: 1,
            reason: "cannot parse Vite config".to_string(),
        },
    }
}

fn extract(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
    allow_imported_include: bool,
) -> ParsedConfig {
    let Some(object) = default_object(program) else {
        return unanalyzable(1, "Vite config has no static default export object");
    };
    let optimize = match property(object, "optimizeDeps") {
        Prop::Missing => return ParsedConfig::Includes(Vec::new()),
        Prop::Dynamic => {
            return unanalyzable(
                line_of(source, object.span.start),
                "optimizeDeps is not a static object",
            );
        }
        Prop::Found(Expression::ObjectExpression(object)) => object.as_ref(),
        Prop::Found(value) => {
            return unanalyzable(
                line_of(source, value.span().start),
                "optimizeDeps is not a static object",
            );
        }
    };
    match property(optimize, "include") {
        Prop::Missing => ParsedConfig::Includes(Vec::new()),
        Prop::Dynamic => unanalyzable(
            line_of(source, optimize.span.start),
            "optimizeDeps is not a static object",
        ),
        Prop::Found(value) => match unwrap_ts_wrappers(value) {
            Expression::ArrayExpression(array) => {
                let parsed = string_entries(source, array.as_ref());
                if allow_imported_include && matches!(parsed, ParsedConfig::Unanalyzable { .. }) {
                    ParsedConfig::Includes(Vec::new())
                } else {
                    parsed
                }
            }
            value => {
                if allow_imported_include {
                    ParsedConfig::Includes(Vec::new())
                } else {
                    unanalyzable(
                        line_of(source, value.span().start),
                        "optimizeDeps.include is not a static array of strings",
                    )
                }
            }
        },
    }
}

pub(super) fn includes_in_array(
    source: &str,
    include: &oxc_ast::ast::ArrayExpression<'_>,
) -> Result<Vec<NestedInclude>, (usize, String)> {
    match string_entries(source, include) {
        ParsedConfig::Includes(entries) => Ok(entries),
        ParsedConfig::Unanalyzable { line, reason } => Err((line, reason)),
    }
}

fn string_entries(source: &str, include: &oxc_ast::ast::ArrayExpression<'_>) -> ParsedConfig {
    let mut entries = Vec::new();
    for element in &include.elements {
        let Some((text, start)) = array_string(element) else {
            return unanalyzable(
                line_of(source, include.span.start),
                "optimizeDeps.include is not a static array of strings",
            );
        };
        for (parent_raw, child_raw) in nested_pairs(&text) {
            let (Some(parent), Some(child)) =
                (package_name(&parent_raw), coverage_name(&child_raw))
            else {
                continue;
            };
            entries.push(NestedInclude {
                parent,
                child,
                line: line_of(source, start),
                text: format!("{parent_raw} > {child_raw}"),
            });
        }
    }
    ParsedConfig::Includes(entries)
}

fn unanalyzable(line: usize, reason: &str) -> ParsedConfig {
    ParsedConfig::Unanalyzable {
        line,
        reason: reason.to_string(),
    }
}

enum Prop<'a> {
    Missing,
    Found(&'a Expression<'a>),
    Dynamic,
}

fn property<'a>(object: &'a ObjectExpression<'a>, name: &str) -> Prop<'a> {
    let mut found = None;
    for prop in &object.properties {
        match prop {
            ObjectPropertyKind::SpreadProperty(_) => return Prop::Dynamic,
            ObjectPropertyKind::ObjectProperty(prop) => {
                if property_key_name(&prop.key).as_deref() == Some(name) {
                    found = Some(&prop.value);
                }
            }
        }
    }
    found.map_or(Prop::Missing, Prop::Found)
}

fn array_string(element: &ArrayExpressionElement<'_>) -> Option<(String, u32)> {
    match element {
        ArrayExpressionElement::StringLiteral(literal) => {
            Some((js_literal(literal).into_owned(), literal.span.start))
        }
        ArrayExpressionElement::TemplateLiteral(template)
            if template.expressions.is_empty() && template.quasis.len() == 1 =>
        {
            let quasi = template.quasis.first()?;
            Some((template_quasi_text(quasi).into_owned(), template.span.start))
        }
        _ => None,
    }
}

fn line_of(source: &str, start: u32) -> usize {
    byte_offset_to_line(source, start as usize) as usize
}

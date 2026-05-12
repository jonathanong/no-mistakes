use crate::ast;
use anyhow::Result;
use oxc_ast::ast::{Argument, CallExpression, Expression};
use oxc_ast_visit::{walk, Visit};
use std::collections::BTreeSet;
use std::path::Path;

/// Extract local URL string literals navigated to in a Playwright test file.
#[cfg(test)]
pub fn extract_playwright_urls(source: &str) -> Vec<String> {
    extract_playwright_url_literals_with_helpers(source, &[])
        .into_iter()
        .filter(|url| url.starts_with('/'))
        .collect()
}

#[cfg(test)]
pub fn extract_playwright_url_literals_with_helpers(
    source: &str,
    navigation_helpers: &[String],
) -> Vec<String> {
    extract_playwright_url_literals_from_path(Path::new("fixture.ts"), source, navigation_helpers)
        .expect("fixture should parse")
}

pub fn extract_playwright_url_literals_from_path(
    path: &Path,
    source: &str,
    navigation_helpers: &[String],
) -> Result<Vec<String>> {
    ast::with_program(path, source, |program, source| {
        let mut visitor = UrlVisitor {
            source,
            navigation_helpers,
            urls: BTreeSet::new(),
        };
        visitor.visit_program(program);
        visitor.urls.into_iter().collect()
    })
}

fn is_candidate_url(url: &str) -> bool {
    url.starts_with('/') || url.starts_with("http://") || url.starts_with("https://")
}

/// Parse `a[href="/users/42"]` to `/users/42`.
fn extract_href_from_selector(selector: &str) -> Option<String> {
    let quoted = selector
        .split("href=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next());
    let single_quoted = selector
        .split("href='")
        .nth(1)
        .and_then(|rest| rest.split('\'').next());
    let url = quoted.or(single_quoted)?;
    if is_candidate_url(url) {
        Some(url.to_string())
    } else {
        None
    }
}

struct UrlVisitor<'a, 'h> {
    source: &'a str,
    navigation_helpers: &'h [String],
    urls: BTreeSet<String>,
}

impl<'a> Visit<'a> for UrlVisitor<'a, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let callee = expression_path(&call.callee);
        let callee_name = callee.as_deref().map(|parts| parts.join("."));

        if callee_ends_with(&callee, "goto") {
            if let Some(url) = call
                .arguments
                .first()
                .and_then(|arg| argument_literal(arg, self.source))
            {
                if is_candidate_url(&url) {
                    self.urls.insert(url);
                }
            }
        } else if callee_ends_with(&callee, "click") {
            if let Some(selector) = call
                .arguments
                .first()
                .and_then(|arg| argument_literal(arg, self.source))
            {
                if let Some(url) = extract_href_from_selector(&selector) {
                    self.urls.insert(url);
                }
            }
        } else if callee_ends_with(&callee, "toHaveURL")
            || callee_name
                .as_deref()
                .is_some_and(|name| self.navigation_helpers.iter().any(|helper| helper == name))
        {
            if let Some(url) = first_candidate_literal(&call.arguments, self.source) {
                self.urls.insert(url);
            }
        }

        walk::walk_call_expression(self, call);
    }
}

fn callee_ends_with(callee: &Option<Vec<String>>, method: &str) -> bool {
    callee
        .as_ref()
        .and_then(|parts| parts.last())
        .is_some_and(|last| last == method)
}

fn first_candidate_literal(arguments: &[Argument<'_>], source: &str) -> Option<String> {
    let mut visitor = LiteralVisitor {
        source,
        literals: Vec::new(),
    };
    for argument in arguments {
        visitor.visit_argument(argument);
    }
    visitor
        .literals
        .into_iter()
        .find(|url| is_candidate_url(url))
}

struct LiteralVisitor<'a> {
    source: &'a str,
    literals: Vec<String>,
}

impl<'a> Visit<'a> for LiteralVisitor<'a> {
    fn visit_string_literal(&mut self, literal: &oxc_ast::ast::StringLiteral<'a>) {
        self.literals.push(literal.value.to_string());
    }

    fn visit_template_literal(&mut self, template: &oxc_ast::ast::TemplateLiteral<'a>) {
        self.literals
            .push(ast::template_literal_text(template, self.source));
        walk::walk_template_literal(self, template);
    }
}

fn argument_literal(argument: &Argument<'_>, source: &str) -> Option<String> {
    match argument {
        Argument::StringLiteral(literal) => Some(literal.value.to_string()),
        Argument::TemplateLiteral(template) => Some(ast::template_literal_text(template, source)),
        _ => None,
    }
}

fn expression_path(expression: &Expression<'_>) -> Option<Vec<String>> {
    match expression {
        Expression::Identifier(identifier) => Some(vec![identifier.name.to_string()]),
        Expression::StaticMemberExpression(member) => {
            let mut parts = expression_path(&member.object).unwrap_or_default();
            parts.push(member.property.name.to_string());
            Some(parts)
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            expression_path(&parenthesized.expression)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fixture_source;

    #[test]
    fn extracts_page_goto_url() {
        let src = fixture_source(&["playwright_urls", "page-goto.ts"]);
        let urls = extract_playwright_urls(&src);
        assert_eq!(urls, vec!["/users/42"]);
    }

    #[test]
    fn extracts_click_href_selector() {
        let src = fixture_source(&["playwright_urls", "click-href.ts"]);
        let urls = extract_playwright_urls(&src);
        assert_eq!(urls, vec!["/dashboard"]);
    }

    #[test]
    fn extracts_double_quoted_goto_and_backtick_single_quoted_href() {
        let src = fixture_source(&["playwright_urls", "quoted-goto-click.ts"]);
        let urls = extract_playwright_urls(&src);
        assert_eq!(urls, vec!["/double", "/single"]);
    }

    #[test]
    fn deduplicates_urls() {
        let src = fixture_source(&["playwright_urls", "duplicate-goto.ts"]);
        let urls = extract_playwright_urls(&src);
        assert_eq!(urls, vec!["/users/1"]);
    }

    #[test]
    fn ignores_external_urls() {
        let src = fixture_source(&["playwright_urls", "external-urls.ts"]);
        let urls = extract_playwright_urls(&src);
        assert!(urls.is_empty());
    }

    #[test]
    fn ignores_non_href_selectors() {
        let src = fixture_source(&["playwright_urls", "non-href-click.ts"]);
        let urls = extract_playwright_urls(&src);
        assert!(urls.is_empty());
    }

    #[test]
    fn ignores_non_url_href_selector() {
        let src = fixture_source(&["playwright_urls", "non-url-href-click.ts"]);
        let urls = extract_playwright_urls(&src);
        assert!(urls.is_empty());
    }

    #[test]
    fn empty_file_returns_empty() {
        let urls = extract_playwright_urls("");
        assert!(urls.is_empty());
    }

    #[test]
    fn extracts_configured_navigation_helper_urls() {
        let src = fixture_source(&["playwright_urls", "navigation-helpers.ts"]);
        let urls = extract_playwright_url_literals_with_helpers(
            &src,
            &["navigateTo".to_string(), "testHelpers.openPath".to_string()],
        );
        assert_eq!(urls, vec!["/profile", "/settings"]);
    }

    #[test]
    fn helper_url_extraction_skips_non_url_literals() {
        let src = fixture_source(&["playwright_urls", "helper-nested-url.ts"]);
        let urls = extract_playwright_url_literals_with_helpers(&src, &["navigateTo".to_string()]);
        assert_eq!(urls, vec!["/dynamic", "/fallback"]);
    }

    #[test]
    fn extracts_to_have_url_assertion_paths() {
        let src = fixture_source(&["playwright_urls", "to-have-url.ts"]);
        let urls = extract_playwright_urls(&src);
        assert_eq!(
            urls,
            vec!["/settings", "/user/${username}/rss-feed-items/viewed"]
        );
    }

    #[test]
    fn to_have_url_uses_first_url_literal_argument() {
        let src = fixture_source(&["playwright_urls", "to-have-url-label.ts"]);
        let urls = extract_playwright_url_literals_with_helpers(&src, &[]);
        assert_eq!(urls, vec!["/settings"]);
    }

    #[test]
    fn parenthesized_callee_is_supported() {
        let src = fixture_source(&["playwright_urls", "parenthesized-callee.ts"]);
        let urls = extract_playwright_urls(&src);
        assert_eq!(urls, vec!["/settings"]);
    }
}

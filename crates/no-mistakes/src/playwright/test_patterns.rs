//! Static Playwright file matchers shared by runner catalogs and selector discovery.
use crate::codebase::ts_source::unwrap_ts_wrappers;
use ::regex::Regex;
use oxc_ast::ast::{ArrayExpressionElement, Expression};

mod regex;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub(crate) struct Patterns {
    pub(crate) globs: Vec<String>,
    pub(crate) regexes: Vec<Regex>,
    pub(crate) warnings: Vec<PatternWarning>,
}

impl Patterns {
    pub(crate) fn parse(
        expression: &Expression<'_>,
        source: &str,
        path: &Path,
        name: &str,
        property_start: u32,
    ) -> Self {
        let offset = property_start as usize;
        let line = source[..offset.min(source.len())]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1;
        let location = PatternWarning {
            path: path.to_path_buf(),
            line,
            property: name.to_string(),
        };
        let mut result = Self::default();
        let expression = unwrap_ts_wrappers(expression);
        if let Expression::ArrayExpression(array) = expression {
            for element in &array.elements {
                match element {
                    ArrayExpressionElement::SpreadElement(_)
                    | ArrayExpressionElement::Elision(_) => {
                        result.unsupported(&location);
                    }
                    _ => result.literal(element.to_expression(), source, &location),
                }
            }
        } else {
            result.literal(expression, source, &location);
        }
        result
    }

    fn literal(&mut self, expression: &Expression<'_>, source: &str, location: &PatternWarning) {
        match unwrap_ts_wrappers(expression) {
            Expression::StringLiteral(value) => {
                self.glob(crate::ast::js_literal(value).into_owned(), location)
            }
            Expression::TemplateLiteral(value) if value.expressions.is_empty() => {
                self.glob(crate::ast::template_literal_text(value, source), location);
            }
            Expression::RegExpLiteral(value) => {
                match regex::compile(value.regex.pattern.text.as_str(), value.regex.flags) {
                    Some(regex) => self.regexes.push(regex),
                    None => self.unsupported(location),
                }
            }
            _ => self.unsupported(location),
        }
    }

    fn glob(&mut self, value: String, location: &PatternWarning) {
        if globset::Glob::new(&crate::codebase::glob_normalize::normalize(&value)).is_ok() {
            self.globs.push(value);
        } else {
            self.unsupported(location);
        }
    }

    fn unsupported(&mut self, location: &PatternWarning) {
        self.warnings.push(location.clone());
    }

    pub(crate) fn extend(&mut self, other: Self) {
        self.globs.extend(other.globs);
        self.regexes.extend(other.regexes);
        self.warnings.extend(other.warnings);
    }

    pub(crate) fn regex_matches(&self, path: &Path) -> bool {
        let absolute = crate::playwright::fsutil::slash_path(path);
        self.regexes.iter().any(|regex| regex.is_match(&absolute))
    }
}

/// Extra matching information alongside the legacy public string-glob fields.
#[derive(Clone, Default)]
pub(crate) struct PatternOptions {
    pub(crate) test_match: Patterns,
    pub(crate) test_ignore: Patterns,
}

#[derive(Clone, Debug)]
pub(crate) struct PatternWarning {
    path: PathBuf,
    line: usize,
    property: String,
}

impl PatternWarning {
    pub(crate) fn render(&self, root: &Path) -> String {
        let path = crate::codebase::ts_source::relative_slash_path(root, &self.path);
        format!("{path}:{}: Playwright {} cannot be fully analyzed statically; test selection and coverage are uncertain for this project. Use string globs or supported RegExp literals (without lookaround, backreferences, Unicode character classes, identity escapes, or v-mode).", self.line, self.property)
    }
}

#[cfg(test)]
mod boundary_tests;
#[cfg(test)]
mod tests;

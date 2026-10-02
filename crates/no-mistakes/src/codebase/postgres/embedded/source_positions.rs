use super::{placeholders, tags};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, TemplateLiteral};
use oxc_span::GetSpan;

mod escapes;
#[cfg(test)]
mod tests;

/// A recovered SQL position whose physical source-line offset changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedSqlSourcePosition {
    pub sql_line: u32,
    pub sql_column: u32,
    pub source_line: u32,
}

pub(super) fn for_expression(
    expr: &Expression<'_>,
    source: &str,
    start: usize,
    line: u32,
) -> Vec<EmbeddedSqlSourcePosition> {
    for_expression_with_offset(expr, source, start, line, 0)
}

pub(super) fn for_expression_with_offset(
    expr: &Expression<'_>,
    source: &str,
    start: usize,
    line: u32,
    placeholder_offset: usize,
) -> Vec<EmbeddedSqlSourcePosition> {
    let expr = unwrap_ts_wrappers(expr);
    let mut out = Positions {
        origin: line,
        placeholder_offset,
        ..Positions::default()
    };
    let line = line + newlines(&source[start..expr.span().start as usize]);
    match expr {
        Expression::StringLiteral(literal) => {
            let raw = &source[literal.span.start as usize + 1..literal.span.end as usize - 1];
            out.append(raw, literal.value.as_str(), line, false);
        }
        Expression::TemplateLiteral(template) => {
            template_positions(template, source, line, &mut out, false)
        }
        Expression::TaggedTemplateExpression(tagged) => {
            let line = line
                + newlines(&source[tagged.span.start as usize..tagged.quasi.span.start as usize]);
            template_positions(
                &tagged.quasi,
                source,
                line,
                &mut out,
                tags::is_string_raw_tag_spelling(&tagged.tag),
            );
        }
        _ => {}
    }
    out.positions
}

fn template_positions(
    template: &TemplateLiteral<'_>,
    source: &str,
    mut line: u32,
    out: &mut Positions,
    raw_mode: bool,
) {
    let mut end = template.span.start as usize;
    for (index, quasi) in template.quasis.iter().enumerate() {
        let start = quasi.span.start as usize;
        if index > 0 {
            let expression_start = template.expressions[index - 1].span().start as usize;
            out.placeholder(index, line + newlines(&source[end..expression_start]));
        }
        line += newlines(&source[end..start]);
        let raw = &source[start..quasi.span.end as usize];
        let cooked = quasi.value.cooked.as_ref().map(|value| value.as_str());
        let decoded = if raw_mode {
            quasi.value.raw.as_str()
        } else {
            cooked.unwrap_or(quasi.value.raw.as_str())
        };
        out.append(raw, decoded, line, raw_mode || cooked.is_none());
        line += newlines(raw);
        end = quasi.span.end as usize;
    }
}

fn newlines(text: &str) -> u32 {
    text.bytes().filter(|byte| *byte == b'\n').count() as u32
}

struct Positions {
    positions: Vec<EmbeddedSqlSourcePosition>,
    line: u32,
    column: u32,
    origin: u32,
    placeholder_offset: usize,
}
impl Default for Positions {
    fn default() -> Self {
        Self {
            positions: Vec::new(),
            line: 1,
            column: 1,
            origin: 1,
            placeholder_offset: 0,
        }
    }
}
impl Positions {
    fn record(&mut self, source_line: u32) {
        let expected = self
            .positions
            .last()
            .map(|position| position.source_line + self.line - position.sql_line)
            .unwrap_or(self.origin + self.line - 1);
        if expected != source_line {
            self.positions.push(EmbeddedSqlSourcePosition {
                sql_line: self.line,
                sql_column: self.column,
                source_line,
            });
        }
    }
    fn placeholder(&mut self, index: usize, line: u32) {
        let index = index + self.placeholder_offset;
        self.record(line);
        self.column += placeholders::PLACEHOLDER_MARKER.len() as u32 + index.ilog10() + 1;
    }
    fn append(&mut self, raw: &str, decoded: &str, mut source_line: u32, raw_mode: bool) {
        if !raw.contains('\n') || raw == decoded {
            self.append_bulk(decoded, source_line, raw == decoded);
            return;
        }
        let mut at = 0;
        let mut extra = 0;
        let mut characters = decoded.chars();
        while let Some(character) = characters.next() {
            if extra == 0 && !raw_mode {
                while let Some(width) = escapes::continuation(&raw[at..]) {
                    source_line += newlines(&raw[at..at + width]);
                    at += width;
                }
            }
            self.record(source_line);
            // Ordinary ASCII surrounding an escape has identical cooked width.
            let run = raw[at..]
                .bytes()
                .take_while(|byte| byte.is_ascii() && !matches!(byte, b'\\' | b'\n' | b'\r'))
                .count();
            if extra == 0 && run > 1 {
                characters.nth(run - 2);
                at += run;
                self.column += run as u32;
                continue;
            }
            if extra > 0 {
                extra -= 1;
            } else {
                let width = escapes::width(&raw[at..], raw_mode);
                extra = escapes::extra_characters(&raw[at..], width, raw_mode);
                source_line += newlines(&raw[at..at + width]);
                at += width;
            }
            if character == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
    }

    fn append_bulk(&mut self, decoded: &str, mut source_line: u32, physical: bool) {
        self.record(source_line);
        for segment in decoded.split_inclusive('\n') {
            if segment.ends_with('\n') {
                self.line += 1;
                self.column = 1;
                if physical {
                    source_line += 1;
                }
                self.record(source_line);
            } else {
                self.column += segment.chars().count() as u32;
            }
        }
    }
}

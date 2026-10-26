//! Directive syntax is interpreted only within Oxc's actual comment records.
use super::options::DEFAULT_DIRECTIVES;
use oxc_ast::ast::Program;
mod syntax;

#[derive(Clone, Debug)]
pub(crate) struct Suppression {
    pub directive: String,
    pub rules: Vec<String>,
    pub line: usize,
}

pub(crate) fn collect(program: &Program<'_>, source: &str) -> Vec<Suppression> {
    let lines = line_starts(source);
    program
        .comments
        .iter()
        .filter_map(|comment| {
            let span = comment.content_span();
            let raw = source.get(span.start as usize..span.end as usize)?;
            let text = raw.trim_start();
            // Leading prose and JSDoc stars are not lint directive headers.
            let lint = DEFAULT_DIRECTIVES
                .iter()
                .copied()
                .find(|name| syntax::header(text, name));
            let (directive, text) = if let Some(directive) = lint {
                (directive, text)
            } else {
                if !comment.is_block() && !matches!(comment.kind, oxc_ast::ast::CommentKind::Line) {
                    return None;
                }
                let text = syntax::typescript_text(raw, comment.is_block());
                let directive = ["@ts-ignore", "@ts-expect-error"]
                    .into_iter()
                    .find(|name| text.starts_with(name))?;
                (directive, text)
            };
            let at = span.start as usize + raw.len() - text.len();
            let tail = text.strip_prefix(directive)?;
            let mut rules: Vec<_> = if directive.starts_with('@') {
                Vec::new()
            } else {
                syntax::without_reason(tail)
                    .split(',')
                    .map(syntax::rule_id)
                    .filter(|v| !v.is_empty())
                    .map(str::to_owned)
                    .collect()
            };
            rules.sort();
            rules.dedup();
            Some(Suppression {
                directive: directive.into(),
                rules,
                line: lines.partition_point(|start| *start <= at),
            })
        })
        .collect()
}

fn line_starts(source: &str) -> Vec<usize> {
    let mut lines = vec![0];
    for (at, c) in source.char_indices() {
        match c {
            '\r' => lines.push(
                at + if source.as_bytes().get(at + 1) == Some(&b'\n') {
                    2
                } else {
                    1
                },
            ),
            '\n' if at == 0 || source.as_bytes()[at - 1] != b'\r' => lines.push(at + 1),
            '\u{2028}' | '\u{2029}' => lines.push(at + c.len_utf8()),
            _ => {}
        }
    }
    lines
}

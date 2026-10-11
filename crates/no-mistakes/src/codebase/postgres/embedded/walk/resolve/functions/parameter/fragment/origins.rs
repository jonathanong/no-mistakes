use crate::codebase::postgres::embedded::{placeholders, source_positions, tags};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

/// Captured with each verified static append during the helper summary pass.
pub(super) fn collect(expr: &Expression<'_>, source: &str, sql: &str) -> Vec<u32> {
    let Expression::TaggedTemplateExpression(tagged) = unwrap_ts_wrappers(expr) else {
        return source_positions::origins::expression(expr, source, sql);
    };
    let raw_mode = tags::is_string_raw_tag_spelling(&tagged.tag);
    let mut out = Vec::with_capacity(sql.len());
    for (index, quasi) in tagged.quasi.quasis.iter().enumerate() {
        if index > 0 {
            out.extend(std::iter::repeat_n(
                tagged.quasi.expressions[index - 1].span().start,
                placeholders::internal_placeholder(index).len(),
            ));
        }
        let raw = &source[quasi.span.start as usize..quasi.span.end as usize];
        out.extend(source_positions::origins::piece(
            raw,
            tags::quasi_text(quasi, raw_mode),
            quasi.span.start,
            raw_mode || quasi.value.cooked.is_none(),
        ));
    }
    out
}

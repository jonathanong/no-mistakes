use super::super::{locations::Locations, types::PostgresSqlSpan};
use super::tokens::{enclosing, is_referential_action, next_significant, output, top_level, word};
use sqlparser::{
    ast::{ColumnDef, Spanned, TableConstraint},
    tokenizer::TokenWithSpan,
};

pub(in crate::codebase::postgres::source) fn table_span(
    value: &TableConstraint,
    tokens: &[TokenWithSpan],
    ranges: &[super::tokens::Range],
    locations: &Locations<'_>,
) -> Option<PostgresSqlSpan> {
    let range = enclosing(ranges, tokens, value.span())?;
    output(range, tokens, locations)
}

pub(in crate::codebase::postgres::source) fn column_span(
    value: &ColumnDef,
    tokens: &[TokenWithSpan],
    ranges: &[super::tokens::Range],
    locations: &Locations<'_>,
) -> Vec<Option<PostgresSqlSpan>> {
    let Some(range) = enclosing(ranges, tokens, value.name.span) else {
        return vec![None; value.options.len()];
    };
    column_option_spans(value, tokens, range, locations)
}

pub(in crate::codebase::postgres::source) fn column_span_in_range(
    value: &ColumnDef,
    tokens: &[TokenWithSpan],
    range: super::tokens::Range,
    locations: &Locations<'_>,
) -> Vec<Option<PostgresSqlSpan>> {
    let Some(segment) = tokens.get(range.0..range.1) else {
        return vec![None; value.options.len()];
    };
    if !segment.iter().any(|token| {
        token.span.start <= value.name.span.start && value.name.span.start <= token.span.end
    }) {
        return vec![None; value.options.len()];
    }
    column_option_spans(value, tokens, range, locations)
}

fn column_option_spans(
    value: &ColumnDef,
    tokens: &[TokenWithSpan],
    range: super::tokens::Range,
    locations: &Locations<'_>,
) -> Vec<Option<PostgresSqlSpan>> {
    use sqlparser::ast::{ColumnOption, Spanned};
    let mut starts = vec![None; value.options.len()];
    let top_level = top_level(tokens, range);
    let mut cursor = range.0;
    for (option_index, option) in value.options.iter().enumerate() {
        let expected = match &option.option {
            ColumnOption::PrimaryKey(_) => "PRIMARY",
            ColumnOption::Unique(_) => "UNIQUE",
            ColumnOption::ForeignKey(_) => "REFERENCES",
            ColumnOption::Check(_) => "CHECK",
            ColumnOption::Default(_) => "DEFAULT",
            ColumnOption::NotNull => "NOT",
            ColumnOption::Null => "NULL",
            ColumnOption::Generated { .. } => "GENERATED",
            _ => continue,
        };
        let ast_span = option.option.span();
        let anchor = ast_span.start;
        let mut candidates = (cursor..range.1).filter(|position| {
            let token = &tokens[*position];
            let in_other_payload = value
                .options
                .iter()
                .enumerate()
                .any(|(other_index, other)| {
                    other_index != option_index && {
                        let span = other.option.span();
                        span.start < span.end
                            && span.start <= token.span.start
                            && token.span.start < span.end
                    }
                });
            let matches = word(token, expected)
                && top_level.contains(position)
                && !is_referential_action(tokens, *position, range.0)
                && (expected != "PRIMARY"
                    || next_significant(tokens, *position + 1, range.1)
                        .is_some_and(|next| word(&tokens[next], "KEY")))
                && (expected != "NOT"
                    || next_significant(tokens, *position + 1, range.1)
                        .is_some_and(|next| word(&tokens[next], "NULL")));
            let follows_name = option
                .name
                .as_ref()
                .is_some_and(|name| token.span.start >= name.span.end);
            matches
                && !in_other_payload
                && if option.name.is_some() {
                    follows_name
                } else {
                    ast_span.start == ast_span.end || token.span.start <= anchor
                }
        });
        let candidate = candidates.next();
        let Some(mut start) = candidate else {
            continue;
        };
        if option.name.is_some() {
            if let Some(named) = (cursor..start).rev().find(|position| {
                word(&tokens[*position], "CONSTRAINT")
                    && next_significant(tokens, *position + 1, start).is_some_and(|name| {
                        option.name.as_ref().is_some_and(|ident| {
                            tokens[name].span.start == ident.span.start
                                && tokens[name].span.end == ident.span.end
                        })
                    })
            }) {
                start = named;
            } else {
                continue;
            }
        }
        starts[option_index] = Some(start);
        cursor = start + 1;
    }
    let mut result = vec![None; value.options.len()];
    let supported = value
        .options
        .iter()
        .map(|option| {
            matches!(
                option.option,
                ColumnOption::PrimaryKey(_)
                    | ColumnOption::Unique(_)
                    | ColumnOption::ForeignKey(_)
                    | ColumnOption::Check(_)
                    | ColumnOption::Default(_)
                    | ColumnOption::NotNull
                    | ColumnOption::Null
                    | ColumnOption::Generated { .. }
            )
        })
        .collect::<Vec<_>>();
    for (option_index, option) in value.options.iter().enumerate() {
        if !matches!(
            option.option,
            ColumnOption::PrimaryKey(_)
                | ColumnOption::Unique(_)
                | ColumnOption::ForeignKey(_)
                | ColumnOption::Check(_)
        ) {
            continue;
        }
        let Some(start) = starts[option_index] else {
            continue;
        };
        if !supported.iter().skip(option_index + 1).all(|known| *known) {
            continue;
        }
        let end = starts
            .iter()
            .skip(option_index + 1)
            .flatten()
            .copied()
            .next()
            .unwrap_or(range.1);
        if let Some(span) = output((start, end), tokens, locations) {
            result[option_index] = Some(span);
        }
    }
    result
}

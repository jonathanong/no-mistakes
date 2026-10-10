use super::scan::{eq, is_dml, is_utility, word_of};
use super::{Ctx, Stop};
use sqlparser::tokenizer::Token;

#[derive(Clone)]
pub(super) struct LabelId {
    quoted: bool,
    value: String,
}

pub(super) enum OpenLabel {
    Absent,
    Valid(LabelId),
    Invalid,
}

pub(super) fn stopped(ctx: &Ctx<'_>, stop: Stop) -> bool {
    match stop {
        Stop::Outer => at_any(ctx, &["END", "EXCEPTION"]),
        Stop::IfBranch => at_any(ctx, &["ELSE", "ELSIF", "ELSEIF", "END"]),
        Stop::Loop => at_word(ctx, "END"),
        Stop::Handler => at_any(ctx, &["END", "WHEN"]),
        Stop::CaseBranch => at_any(ctx, &["WHEN", "ELSE", "END"]),
        Stop::None => false,
    }
}

pub(super) fn at_plain_end(ctx: &Ctx<'_>) -> bool {
    at_word(ctx, "END") && !end_modifier(ctx)
}

fn end_modifier(ctx: &Ctx<'_>) -> bool {
    let index = peek_index(ctx).expect("END");
    ((index + 1)..ctx.tokens.len())
        .find(|index| !matches!(ctx.tokens[*index].token, Token::Whitespace(_)))
        .and_then(|index| word_of(&ctx.tokens[index].token))
        .is_some_and(|word| matches!(word, "IF" | "LOOP" | "CASE"))
}

pub(super) fn at_dml(ctx: &Ctx<'_>) -> bool {
    peek_word(ctx).is_some_and(is_dml)
}

pub(super) fn at_utility(ctx: &Ctx<'_>) -> bool {
    peek_word(ctx).is_some_and(is_utility)
}

pub(super) fn at_any(ctx: &Ctx<'_>, words: &[&str]) -> bool {
    peek_word(ctx).is_some_and(|word| words.iter().any(|value| eq(word, value)))
}

pub(super) fn at_word(ctx: &Ctx<'_>, value: &str) -> bool {
    peek_word(ctx).is_some_and(|word| eq(word, value))
}

pub(super) fn eat_word(ctx: &mut Ctx<'_>, value: &str) -> bool {
    if at_word(ctx, value) {
        bump(ctx);
        true
    } else {
        false
    }
}

pub(super) fn eat_semi(ctx: &mut Ctx<'_>) -> bool {
    let Some(index) = peek_index(ctx) else {
        return false;
    };
    if matches!(ctx.tokens[index].token, Token::SemiColon) {
        ctx.index = index + 1;
        true
    } else {
        false
    }
}

pub(super) fn eat_label(ctx: &mut Ctx<'_>) -> Option<LabelId> {
    let index = peek_index(ctx)?;
    // Closing labels are any identifier. Quoted words stay out of `word_of`.
    let label = label_id(&ctx.tokens[index].token)?;
    let semicolon = ((index + 1)..ctx.tokens.len())
        .find(|index| !matches!(ctx.tokens[*index].token, Token::Whitespace(_)))
        .is_some_and(|index| matches!(ctx.tokens[index].token, Token::SemiColon));
    if semicolon {
        bump(ctx);
        Some(label)
    } else {
        None
    }
}

pub(super) fn skip_label(ctx: &mut Ctx<'_>) -> OpenLabel {
    let Some(index) = peek_index(ctx) else {
        return OpenLabel::Absent;
    };
    // `<<>>` is one custom operator in the PostgreSQL dialect, not two shift tokens.
    if matches!(&ctx.tokens[index].token, Token::CustomBinaryOperator(operator) if operator == "<<>>")
    {
        ctx.index = index + 1;
        return OpenLabel::Invalid;
    }
    if !matches!(ctx.tokens[index].token, Token::ShiftLeft) {
        return OpenLabel::Absent;
    }
    ctx.index = index + 1;
    let mut found = None;
    let mut invalid = false;
    let mut closed = false;
    while let Some(index) = peek_index(ctx) {
        if matches!(ctx.tokens[index].token, Token::ShiftRight) {
            ctx.index = index + 1;
            closed = true;
            break;
        }
        let label = label_id(&ctx.tokens[index].token);
        ctx.index = index + 1;
        match label {
            Some(label) if found.is_none() && !invalid => found = Some(label),
            _ => invalid = true,
        }
    }
    if closed && !invalid {
        found.map(OpenLabel::Valid).unwrap_or(OpenLabel::Invalid)
    } else {
        OpenLabel::Invalid
    }
}

pub(super) fn note_label(ctx: &mut Ctx<'_>, open: &OpenLabel, close: Option<&LabelId>) {
    if label_disagrees(open, close) {
        ctx.label_invalid = true;
        if matches!(open, OpenLabel::Valid(_)) && close.is_some() {
            ctx.label_mismatches.push(ctx.index - 1);
        }
    }
}

pub(super) fn record_invalid_label(ctx: &mut Ctx<'_>, open: &OpenLabel) {
    if matches!(open, OpenLabel::Invalid) {
        ctx.label_invalid = true;
    }
}

fn label_disagrees(open: &OpenLabel, close: Option<&LabelId>) -> bool {
    match open {
        OpenLabel::Invalid => true,
        OpenLabel::Valid(open) => close.is_some_and(|close| !same_label(open, close)),
        OpenLabel::Absent => false,
    }
}

fn same_label(open: &LabelId, close: &LabelId) -> bool {
    stored_label(open) == stored_label(close)
}

fn stored_label(label: &LabelId) -> String {
    // PostgreSQL folds unquoted names, then truncates either form to 63 UTF-8 bytes.
    let mut value = if label.quoted {
        label.value.clone()
    } else {
        label.value.to_ascii_lowercase()
    };
    let mut end = value.len().min(63);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value.truncate(end);
    value
}

fn label_id(token: &Token) -> Option<LabelId> {
    match token {
        Token::Word(word) if plpgsql_label_word(word) => Some(LabelId {
            quoted: word.quote_style.is_some(),
            value: word.value.clone(),
        }),
        _ => None,
    }
}

fn plpgsql_label_word(word: &sqlparser::tokenizer::Word) -> bool {
    // PL/pgSQL reserves a smaller, different vocabulary than SQL column names.
    // PostgreSQL src/pl/plpgsql/src/pl_reserved_kwlist.h is the source of truth.
    const RESERVED: &[&str] = &[
        "all", "begin", "by", "case", "declare", "else", "end", "execute", "for", "foreach",
        "from", "if", "in", "into", "loop", "not", "null", "or", "strict", "then", "to", "using",
        "when", "while",
    ];
    word.quote_style.is_some()
        || !RESERVED
            .iter()
            .any(|reserved| word.value.eq_ignore_ascii_case(reserved))
}

pub(super) fn bump(ctx: &mut Ctx<'_>) {
    if let Some(index) = peek_index(ctx) {
        ctx.index = index + 1;
    }
}

pub(super) fn peek_index(ctx: &Ctx<'_>) -> Option<usize> {
    (ctx.index..ctx.tokens.len())
        .find(|index| !matches!(ctx.tokens[*index].token, Token::Whitespace(_)))
}

pub(super) fn peek_word<'a>(ctx: &Ctx<'a>) -> Option<&'a str> {
    peek_index(ctx).and_then(|index| word_of(&ctx.tokens[index].token))
}

pub(super) fn last_index(ctx: &Ctx<'_>, start: usize) -> usize {
    ctx.index
        .saturating_sub(1)
        .max(start)
        .min(ctx.tokens.len().saturating_sub(1))
        .max(start)
}

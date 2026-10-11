use super::super::super::{SqlFactSite as Site, SqlStatementFileFacts, SqlVariantLocations};
use crate::fx::FxHashMap;
use sqlparser::tokenizer::{Token, TokenWithSpan};
mod settings;

pub(super) fn collect(
    facts: &SqlStatementFileFacts,
    tokens: &[TokenWithSpan],
    out: &mut SqlVariantLocations,
) {
    if let Some(token) = tokens
        .iter()
        .find(|token| !matches!(token.token, Token::Whitespace(_) | Token::EOF))
    {
        out.insert(
            Site::Annotation,
            token.span.start.line as usize,
            token.span.start.column as usize,
        );
    }
    let mut seen = FxHashMap::default();
    for (i, fact) in facts.statement_kinds.iter().enumerate() {
        let word = fact.kind.split_whitespace().next().unwrap_or_default();
        let occurrence = seen.entry((word, fact.line)).or_insert(0usize);
        let at = keyword(tokens, word, fact.line, *occurrence);
        *occurrence += 1;
        out.insert(Site::StatementKind(i), at.0, at.1);
    }
    settings::collect(facts, tokens, out);
    for (i, fact) in facts.function_calls.iter().enumerate() {
        out.insert(Site::Function(i), fact.line, fact.column);
    }
    for (i, fact) in facts.offset_uses.iter().enumerate() {
        out.insert(Site::Offset(i), fact.line, fact.column);
    }
    for (i, fact) in facts.limit_uses.iter().enumerate() {
        out.insert(Site::Limit(i), fact.line, fact.column);
    }
    for (i, fact) in facts.sweeps.iter().enumerate() {
        out.insert(Site::Sweep(i), fact.line, fact.column);
    }
    let mut writes = FxHashMap::default();
    for (i, fact) in facts.writes.iter().enumerate() {
        let occurrence = writes.entry(fact.line).or_insert(0usize);
        let at = tokens.iter().filter(|token| token.span.start.line as usize == fact.line && matches!(&token.token, Token::Word(word) if matches!(word.keyword, sqlparser::keywords::Keyword::INSERT | sqlparser::keywords::Keyword::UPDATE | sqlparser::keywords::Keyword::MERGE))).nth(*occurrence).map(|token| (fact.line, token.span.start.column as usize)).unwrap_or((fact.line, 1));
        *occurrence += 1;
        out.insert(Site::Write(i), at.0, at.1);
    }
}
fn keyword(
    tokens: &[TokenWithSpan],
    value: &str,
    line: usize,
    occurrence: usize,
) -> (usize, usize) {
    tokens.iter().filter(|token| token.span.start.line as usize == line && matches!(&token.token, Token::Word(word) if word.value.eq_ignore_ascii_case(value))).nth(occurrence).map(|token| (line, token.span.start.column as usize)).unwrap_or((line, 1))
}

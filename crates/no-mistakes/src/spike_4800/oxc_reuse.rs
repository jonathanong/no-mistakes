pub use super::ast_complexity::{collect_functions, FunctionSpan};

use cpd_core::hash::hash_token;
use cpd_core::models::{DetectionToken, Location, TokenKind};
use oxc_allocator::Allocator;
use oxc_parser::Kind;
use oxc_span::SourceType;
use std::path::Path;

const fn map_oxc_kind(kind: Kind) -> TokenKind {
    if matches!(kind, Kind::Ident) {
        return TokenKind::Identifier;
    }
    if kind.is_any_keyword() {
        return TokenKind::Keyword;
    }
    if kind.is_literal() {
        return TokenKind::Literal;
    }
    if kind.is_assignment_operator()
        || kind.is_binary_operator()
        || kind.is_logical_operator()
        || kind.is_unary_operator()
        || kind.is_update_operator()
        || matches!(kind, Kind::Arrow)
    {
        return TokenKind::Operator;
    }
    match kind {
        Kind::Semicolon
        | Kind::Comma
        | Kind::Dot
        | Kind::Dot3
        | Kind::Colon
        | Kind::LParen
        | Kind::RParen
        | Kind::LCurly
        | Kind::RCurly
        | Kind::LBrack
        | Kind::RBrack
        | Kind::At
        | Kind::QuestionDot => TokenKind::Punctuation,
        _ => TokenKind::Other,
    }
}

pub fn parse_and_tokenize_oxc_0159(
    path: &Path,
    source: &str,
) -> (Vec<DetectionToken>, Vec<FunctionSpan>) {
    let allocator = Allocator::new();
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::tsx());
    let ret = crate::ast::parse_with_tokens(&allocator, source, source_type);
    let functions = collect_functions(&ret.program, source);
    let mut line_starts = vec![0usize];
    for (idx, &b) in source.as_bytes().iter().enumerate() {
        if b == b'\n' {
            line_starts.push(idx + 1);
        }
    }
    let locate = |offset: usize| {
        let line_idx = line_starts
            .partition_point(|&s| s <= offset)
            .saturating_sub(1);
        Location::new(
            (line_idx + 1) as u32,
            (offset - line_starts[line_idx]) as u32,
            offset as u32,
        )
    };
    let mut out = Vec::with_capacity(ret.tokens.len());
    for token in ret.tokens {
        let start = (token.start() as usize).min(source.len());
        let end = (token.end() as usize).min(source.len());
        if start >= end || matches!(token.kind(), Kind::Eof | Kind::Undetermined | Kind::Skip) {
            continue;
        }
        let kind = map_oxc_kind(token.kind());
        let hash = hash_token(kind.discriminant(), &source[start..end], false);
        out.push(DetectionToken {
            hash,
            raw_hash: hash,
            name: 0,
            start: locate(start),
            end: locate(end),
            range: [start, end],
        });
    }
    (out, functions)
}

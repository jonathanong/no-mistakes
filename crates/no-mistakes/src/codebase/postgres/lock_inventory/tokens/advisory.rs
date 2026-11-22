use super::{InventoryLockFact, Lexeme, Segment, Segments};
use crate::codebase::postgres::SqlFunctionCallFact;
use sqlparser::tokenizer::{Location, Token, TokenWithSpan};

impl Segments<'_> {
    pub(in crate::codebase::postgres::lock_inventory) fn advisory(
        &self,
        call: &SqlFunctionCallFact,
        out: &mut Vec<InventoryLockFact>,
    ) -> bool {
        let at = call_location(call);
        self.at(at)
            .is_some_and(|segment| segment.advisory(call, at, out))
    }

    pub(in crate::codebase::postgres::lock_inventory) fn advisories(
        &self,
        calls: &[&SqlFunctionCallFact],
        out: &mut Vec<InventoryLockFact>,
    ) {
        for segment in &self.segments {
            let start = calls.partition_point(|call| call_location(call) < segment.start());
            for call in calls[start..]
                .iter()
                .take_while(|call| call_location(call) < segment.end())
            {
                segment.advisory(call, call_location(call), out);
            }
        }
    }
}

impl Segment<'_> {
    fn advisory(
        &self,
        call: &SqlFunctionCallFact,
        at: Location,
        out: &mut Vec<InventoryLockFact>,
    ) -> bool {
        let tail = &self.tokens[self.index_at(at)..];
        let Some(head) = tail.first().filter(|token| token.at() == at) else {
            return false;
        };
        let mut tokens = tail.iter();
        for (index, name) in call.name_parts.iter().enumerate() {
            if index > 0
                && !tokens
                    .next()
                    .is_some_and(|lexeme| *lexeme.token() == Token::Period)
            {
                return false;
            }
            if !tokens.next().is_some_and(|lexeme| lexeme.ident(name)) {
                return false;
            }
        }
        if !tokens
            .next()
            .is_some_and(|lexeme| *lexeme.token() == Token::LParen)
        {
            return false;
        }
        self.record("advisory-lock", head, out);
        true
    }
}

impl Lexeme<'_> {
    fn ident(&self, expected: &str) -> bool {
        match self.token() {
            Token::Word(value) if value.quote_style.is_none() => {
                value.value.eq_ignore_ascii_case(expected)
            }
            Token::Word(value) => value.value == expected,
            _ => false,
        }
    }
}

pub(super) fn advisory_token(token: &TokenWithSpan) -> bool {
    matches!(&token.token, Token::Word(value)
    if super::advisory_name(&value.value) || value.quote_style.is_none()
        && ["pg_advisory_", "pg_try_advisory_"].iter().any(|prefix| {
            value.value.get(..prefix.len()).is_some_and(|head| head.eq_ignore_ascii_case(prefix))
        }))
}

fn call_location(call: &SqlFunctionCallFact) -> Location {
    Location::new(call.line as u64, call.column as u64)
}

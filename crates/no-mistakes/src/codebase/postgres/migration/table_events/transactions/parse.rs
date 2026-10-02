use super::{Command, Marker};
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub fn parse(tokens: &[&TokenWithSpan], order: Vec<usize>) -> Option<Marker> {
    let first = tokens.first()?;
    let command = if keyword(first, "BEGIN") {
        if keyword_at(tokens, 1, "ATOMIC") {
            return None;
        }
        Some(Command::Begin)
    } else if keyword(first, "START") && keyword_at(tokens, 1, "TRANSACTION") {
        Some(Command::Begin)
    } else if keyword(first, "COMMIT") || keyword(first, "END") {
        if keyword_at(tokens, 1, "PREPARED") {
            return None;
        }
        Some(if and_chain(tokens, transaction_modifier_end(tokens, 1)) {
            Command::CommitAndChain
        } else {
            Command::Commit
        })
    } else if keyword(first, "ROLLBACK") || keyword(first, "ABORT") {
        if keyword_at(tokens, 1, "PREPARED") {
            return None;
        }
        let mut at = 1;
        if keyword_at(tokens, at, "WORK") || keyword_at(tokens, at, "TRANSACTION") {
            at += 1;
        }
        if keyword(first, "ROLLBACK") && keyword_at(tokens, at, "TO") {
            at += 1;
            if keyword_at(tokens, at, "SAVEPOINT") {
                at += 1;
            }
            Some(Command::RollbackTo(identifier_at(tokens, at)?))
        } else {
            Some(if keyword(first, "ROLLBACK") && and_chain(tokens, at) {
                Command::RollbackAndChain
            } else {
                Command::Rollback
            })
        }
    } else if keyword(first, "SAVEPOINT") {
        Some(Command::Savepoint(identifier_at(tokens, 1)?))
    } else if keyword(first, "RELEASE") {
        let at = if keyword_at(tokens, 1, "SAVEPOINT") {
            2
        } else {
            1
        };
        Some(Command::Release(identifier_at(tokens, at)?))
    } else {
        None
    }?;
    Some(Marker { order, command })
}

fn transaction_modifier_end(tokens: &[&TokenWithSpan], mut at: usize) -> usize {
    if keyword_at(tokens, at, "WORK") || keyword_at(tokens, at, "TRANSACTION") {
        at += 1;
    }
    at
}

fn and_chain(tokens: &[&TokenWithSpan], at: usize) -> bool {
    keyword_at(tokens, at, "AND") && keyword_at(tokens, at + 1, "CHAIN")
}

fn keyword_at(tokens: &[&TokenWithSpan], at: usize, expected: &str) -> bool {
    tokens.get(at).is_some_and(|token| keyword(token, expected))
}

fn keyword(token: &TokenWithSpan, expected: &str) -> bool {
    matches!(&token.token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}

fn identifier_at(tokens: &[&TokenWithSpan], at: usize) -> Option<String> {
    let Token::Word(word) = &tokens.get(at)?.token else {
        return None;
    };
    Some(if word.quote_style.is_some() {
        word.value.clone()
    } else {
        word.value.to_ascii_lowercase()
    })
}

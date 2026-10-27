use super::{
    binding_word,
    cursor::{Cursor, Token},
};
use oxc_span::Span;

type Result = std::result::Result<Option<Span>, ()>;

pub(super) fn clause(cursor: &mut Cursor<'_>, import: bool) -> Result {
    let mut quoted_local = false;
    if cursor.peek() == Some(Token::Punctuation('}')) {
        cursor.next();
    } else {
        loop {
            let (bad, quoted) = name(cursor)?;
            if bad.is_some() {
                return Ok(bad);
            }
            quoted_local |= quoted;
            if cursor.peek() == Some(Token::Word("as")) {
                cursor.next();
                if import {
                    binding(cursor)?;
                } else {
                    let (bad, _) = name(cursor)?;
                    if bad.is_some() {
                        return Ok(bad);
                    }
                }
            } else if import && quoted {
                return Err(());
            }
            match cursor.next() {
                Some(Token::Punctuation('}')) => break,
                Some(Token::Punctuation(',')) => {
                    if cursor.peek() == Some(Token::Punctuation('}')) {
                        cursor.next();
                        break;
                    }
                }
                _ => return Err(()),
            }
        }
    }
    if cursor.peek() == Some(Token::Word("from")) {
        cursor.next();
        source(cursor)?;
    } else if import || quoted_local {
        return Err(());
    }
    semicolon(cursor)?;
    Ok(None)
}

pub(super) fn all(cursor: &mut Cursor<'_>) -> Result {
    if cursor.peek() == Some(Token::Word("as")) {
        cursor.next();
        let (bad, _) = name(cursor)?;
        if bad.is_some() {
            return Ok(bad);
        }
    }
    if cursor.next() != Some(Token::Word("from")) {
        return Err(());
    }
    source(cursor)?;
    semicolon(cursor)?;
    Ok(None)
}

fn name(cursor: &mut Cursor<'_>) -> std::result::Result<(Option<Span>, bool), ()> {
    match cursor.next() {
        Some(Token::Word(word)) if binding_word(word) => Ok((None, false)),
        Some(Token::Quoted(span, lone)) => Ok((lone.then_some(span), true)),
        _ => Err(()),
    }
}

fn binding(cursor: &mut Cursor<'_>) -> std::result::Result<(), ()> {
    match cursor.next() {
        Some(Token::Word(word)) if binding_word(word) => Ok(()),
        _ => Err(()),
    }
}

fn source(cursor: &mut Cursor<'_>) -> std::result::Result<(), ()> {
    if matches!(cursor.next(), Some(Token::Quoted(_, _))) {
        Ok(())
    } else {
        Err(())
    }
}

fn semicolon(cursor: &mut Cursor<'_>) -> std::result::Result<(), ()> {
    if cursor.next() == Some(Token::Punctuation(';')) {
        Ok(())
    } else {
        Err(())
    }
}

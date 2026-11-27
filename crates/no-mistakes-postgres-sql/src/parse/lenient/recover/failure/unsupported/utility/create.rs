//! Recognize utility CREATE forms without rewriting their retained source.
use super::{string, Cursor};
use sqlparser::tokenizer::Token;

pub(super) fn valid(cursor: &mut Cursor<'_>) -> bool {
    if cursor.words(&["CREATE", "EXTENSION"]) {
        extension(cursor).is_some()
    } else if cursor.words(&["CREATE", "USER"]) {
        user(cursor)
    } else {
        false
    }
}

pub(crate) fn extension_has_version(tokens: &[&Token]) -> bool {
    let mut cursor = Cursor { tokens, at: 0 };
    cursor.words(&["CREATE", "EXTENSION"]) && extension(&mut cursor) == Some(true)
}

fn extension(cursor: &mut Cursor<'_>) -> Option<bool> {
    if cursor.word("IF") && !cursor.words(&["NOT", "EXISTS"]) {
        return None;
    }
    if !cursor.identifier() {
        return None;
    }
    cursor.word("WITH");
    let mut seen = [false; 3];
    while cursor.at < cursor.tokens.len() {
        let option = if cursor.word("SCHEMA") {
            if !cursor.identifier() {
                return None;
            }
            0
        } else if cursor.word("VERSION") {
            if !(string(cursor) || cursor.identifier()) {
                return None;
            }
            1
        } else if cursor.word("CASCADE") {
            2
        } else {
            return None;
        };
        if std::mem::replace(&mut seen[option], true) {
            return None;
        }
    }
    Some(seen[1])
}

fn user(cursor: &mut Cursor<'_>) -> bool {
    if !cursor.identifier() {
        return false;
    }
    cursor.word("WITH");
    // Only boolean role attributes are recognized here. Other role options
    // require their own complete grammar before they can become advisories.
    let attributes = [
        ["SUPERUSER", "NOSUPERUSER"],
        ["CREATEDB", "NOCREATEDB"],
        ["CREATEROLE", "NOCREATEROLE"],
        ["INHERIT", "NOINHERIT"],
        ["LOGIN", "NOLOGIN"],
        ["REPLICATION", "NOREPLICATION"],
        ["BYPASSRLS", "NOBYPASSRLS"],
    ];
    let mut seen = [false; 7];
    while cursor.at < cursor.tokens.len() {
        let Some(option) = attributes
            .iter()
            .position(|words| words.iter().any(|word| cursor.word(word)))
        else {
            return false;
        };
        if std::mem::replace(&mut seen[option], true) {
            return false;
        }
    }
    true
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;

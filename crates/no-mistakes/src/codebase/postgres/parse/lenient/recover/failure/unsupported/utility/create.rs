//! Recognize utility CREATE forms without rewriting their retained source.
use super::{string, Cursor};

pub(super) fn valid(cursor: &mut Cursor<'_>) -> bool {
    if cursor.words(&["CREATE", "EXTENSION"]) {
        extension(cursor)
    } else if cursor.words(&["CREATE", "USER"]) {
        user(cursor)
    } else {
        false
    }
}

fn extension(cursor: &mut Cursor<'_>) -> bool {
    if cursor.word("IF") && !cursor.words(&["NOT", "EXISTS"]) {
        return false;
    }
    if !cursor.identifier() {
        return false;
    }
    cursor.word("WITH");
    let mut seen = [false; 3];
    while cursor.at < cursor.tokens.len() {
        let option = if cursor.word("SCHEMA") {
            if !cursor.identifier() {
                return false;
            }
            0
        } else if cursor.word("VERSION") {
            if !(string(cursor) || cursor.identifier()) {
                return false;
            }
            1
        } else if cursor.word("CASCADE") {
            2
        } else {
            return false;
        };
        if std::mem::replace(&mut seen[option], true) {
            return false;
        }
    }
    true
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

#[cfg(test)]
mod tests;

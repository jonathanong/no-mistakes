use super::{number, string, Cursor};

pub(super) fn valid(cursor: &mut Cursor<'_>) -> bool {
    if cursor.word("CHECKPOINT") {
        return true;
    }
    if cursor.words(&["PREPARE", "TRANSACTION"])
        || cursor.words(&["COMMIT", "PREPARED"])
        || cursor.words(&["ROLLBACK", "PREPARED"])
    {
        return string(cursor);
    }
    if cursor.word("FETCH") || cursor.word("MOVE") {
        return fetch(cursor);
    }
    if cursor.words(&["RESET", "TIME", "ZONE"]) {
        return true;
    }
    if !cursor.word("SET") {
        return false;
    }
    if cursor.words(&["XML", "OPTION"]) {
        return cursor.word("DOCUMENT") || cursor.word("CONTENT");
    }
    if cursor.word("CONSTRAINTS") {
        return (cursor.word("ALL") || cursor.names())
            && (cursor.word("DEFERRED") || cursor.word("IMMEDIATE"));
    }
    if !cursor.word("TRANSACTION") {
        return false;
    }
    loop {
        let mode = if cursor.words(&["ISOLATION", "LEVEL"]) {
            cursor.word("SERIALIZABLE")
                || cursor.words(&["REPEATABLE", "READ"])
                || cursor.words(&["READ", "COMMITTED"])
                || cursor.words(&["READ", "UNCOMMITTED"])
        } else if cursor.word("READ") {
            cursor.word("ONLY") || cursor.word("WRITE")
        } else {
            cursor.word("DEFERRABLE") || cursor.words(&["NOT", "DEFERRABLE"])
        };
        if !mode {
            return false;
        }
        if !cursor.comma() {
            return true;
        }
    }
}

fn fetch(cursor: &mut Cursor<'_>) -> bool {
    if cursor.word("ABSOLUTE") || cursor.word("RELATIVE") {
        if !number(cursor, true) {
            return false;
        }
    } else if !(["NEXT", "PRIOR", "FIRST", "LAST", "ALL"]
        .iter()
        .any(|word| cursor.word(word))
        || number(cursor, true))
        && (cursor.word("FORWARD") || cursor.word("BACKWARD"))
        && !cursor.word("ALL")
    {
        number(cursor, false);
    }
    if !cursor.word("FROM") {
        cursor.word("IN");
    }
    cursor.identifier()
}

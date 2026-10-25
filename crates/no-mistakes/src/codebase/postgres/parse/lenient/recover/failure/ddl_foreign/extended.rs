use super::cursor::Cursor;
use sqlparser::tokenizer::Token;

pub(super) fn drop_names(c: &mut Cursor<'_>) -> bool {
    if c.words(&["IF", "EXISTS"]) {}
    if !c.names() {
        return false;
    }
    if !c.word("CASCADE") {
        c.word("RESTRICT");
    }
    true
}

pub(super) fn drop_unqualified_names(c: &mut Cursor<'_>) -> bool {
    if c.words(&["IF", "EXISTS"]) {}
    if !c.simple_names() {
        return false;
    }
    if !c.word("CASCADE") {
        c.word("RESTRICT");
    }
    true
}

pub(super) fn drop_mapping(c: &mut Cursor<'_>) -> bool {
    if c.words(&["IF", "EXISTS"]) {}
    c.word("FOR") && c.mapping_role() && c.word("SERVER") && c.identifier()
}

pub(super) fn alter_server(c: &mut Cursor<'_>) -> bool {
    if !c.identifier() {
        return false;
    }
    if c.words(&["OWNER", "TO"]) {
        return c.role();
    }
    if c.words(&["RENAME", "TO"]) {
        return c.identifier();
    }
    let version = c.word("VERSION");
    if version && !c.string() {
        return false;
    }
    let options = c.word("OPTIONS");
    if options && !c.options(true) {
        return false;
    }
    version || options
}

pub(super) fn alter_mapping(c: &mut Cursor<'_>) -> bool {
    c.word("FOR")
        && c.mapping_role()
        && c.word("SERVER")
        && c.identifier()
        && c.word("OPTIONS")
        && c.options(true)
}

pub(super) fn alter_foreign_table(c: &mut Cursor<'_>) -> bool {
    if c.words(&["IF", "EXISTS"]) {}
    let _only = c.word("ONLY");
    if !c.name() {
        return false;
    }
    c.punctuation(Token::Mul);
    if c.word("OPTIONS") {
        return c.options(true);
    }
    if c.words(&["RENAME", "TO"]) {
        return c.identifier();
    }
    if c.words(&["SET", "SCHEMA"]) {
        return c.identifier();
    }
    false
}

pub(super) fn create_foreign_table(c: &mut Cursor<'_>) -> bool {
    if c.words(&["IF", "NOT", "EXISTS"]) {}
    if !c.name() {
        return false;
    }
    if c.left_paren() {
        if !c.right_paren() {
            loop {
                if !c.identifier() || !c.type_name() {
                    return false;
                }
                if c.word("NOT") && !c.word("NULL") {
                    return false;
                }
                if c.word("NULL") {}
                if c.right_paren() {
                    break;
                }
                if !c.comma() {
                    return false;
                }
            }
        }
    } else {
        return false;
    }
    if !c.word("SERVER") || !c.identifier() {
        return false;
    }
    !c.word("OPTIONS") || c.options(false)
}

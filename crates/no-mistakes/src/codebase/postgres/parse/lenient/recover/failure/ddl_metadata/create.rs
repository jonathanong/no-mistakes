use super::Cursor;
use sqlparser::tokenizer::Token;
pub(super) fn parse(c: &mut Cursor<'_>) -> bool {
    if !c.word("CREATE") {
        return false;
    }
    if c.word("GROUP") {
        return c.identifier();
    }
    if c.word("STATISTICS") {
        c.words(&["IF", "NOT", "EXISTS"]);
        if !c.name()
            || !c.word("ON")
            || !c.identifier()
            || !c.token(Token::Comma)
            || !c.identifier()
        {
            return false;
        }
        while c.token(Token::Comma) {
            if !c.identifier() {
                return false;
            }
        }
        return c.word("FROM") && c.name();
    }
    if c.word("SUBSCRIPTION") {
        if !c.identifier()
            || !c.word("CONNECTION")
            || !c.string()
            || !c.word("PUBLICATION")
            || !c.identifier()
        {
            return false;
        }
        while c.token(Token::Comma) {
            if !c.identifier() {
                return false;
            }
        }
        return true;
    }
    if c.word("TABLESPACE") {
        if !c.identifier() {
            return false;
        }
        if c.word("OWNER")
            && !(c.word("CURRENT_ROLE")
                || c.word("CURRENT_USER")
                || c.word("SESSION_USER")
                || c.identifier())
        {
            return false;
        }
        return c.word("LOCATION") && c.string();
    }
    c.words(&["OR", "REPLACE"]);
    if c.word("RULE") {
        return c.identifier()
            && c.words(&["AS", "ON"])
            && (c.word("SELECT") || c.word("INSERT") || c.word("UPDATE") || c.word("DELETE"))
            && c.word("TO")
            && c.name()
            && c.word("DO")
            && {
                if !c.word("ALSO") {
                    c.word("INSTEAD");
                }
                c.word("NOTHING")
            };
    }
    c.word("TRUSTED");
    c.word("PROCEDURAL");
    if !c.word("LANGUAGE") || !c.identifier() {
        return false;
    }
    if c.word("HANDLER") {
        if !c.name() {
            return false;
        }
        if c.word("INLINE") && !c.name() {
            return false;
        }
        if c.word("VALIDATOR") && !c.name() {
            return false;
        }
    }
    true
}

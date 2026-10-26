use super::cursor::Cursor;
#[path = "signatures.rs"]
mod signatures;

pub(super) fn create_fdw(c: &mut Cursor<'_>) -> bool {
    if !c.identifier() {
        return false;
    }
    if c.word("HANDLER") {
        if !c.function_name() {
            return false;
        }
    } else {
        c.words(&["NO", "HANDLER"]);
    }
    if c.word("VALIDATOR") {
        if !c.function_name() {
            return false;
        }
    } else {
        c.words(&["NO", "VALIDATOR"]);
    }
    if c.word("OPTIONS") && !c.options(false) {
        return false;
    }
    true
}

pub(super) fn alter_fdw(c: &mut Cursor<'_>) -> bool {
    if !c.identifier() {
        return false;
    }
    if c.words(&["OWNER", "TO"]) {
        return c.role();
    }
    if c.words(&["RENAME", "TO"]) {
        return c.identifier();
    }
    let mut any = false;
    if c.word("HANDLER") {
        if !c.function_name() {
            return false;
        }
        any = true;
    } else if c.words(&["NO", "HANDLER"]) {
        any = true;
    }
    if c.word("VALIDATOR") {
        if !c.function_name() {
            return false;
        }
        any = true;
    } else if c.words(&["NO", "VALIDATOR"]) {
        any = true;
    }
    if c.word("OPTIONS") {
        if !c.options(true) {
            return false;
        }
        any = true;
    }
    any
}

pub(super) fn create_mapping(c: &mut Cursor<'_>) -> bool {
    c.words(&["IF", "NOT", "EXISTS"]);
    if !c.word("FOR") || !c.mapping_role() || !c.word("SERVER") || !c.identifier() {
        return false;
    }
    !c.word("OPTIONS") || c.options(false)
}

pub(super) fn import_schema(c: &mut Cursor<'_>) -> bool {
    if !c.identifier() {
        return false;
    }
    if c.word("LIMIT") {
        if !c.word("TO") || !c.names_in_parens() {
            return false;
        }
    } else if c.word("EXCEPT") && !c.names_in_parens() {
        return false;
    }
    c.words(&["FROM", "SERVER"])
        && c.identifier()
        && c.word("INTO")
        && c.identifier()
        && (!c.word("OPTIONS") || c.options(false))
}

pub(super) fn security_label(c: &mut Cursor<'_>) -> bool {
    if c.word("FOR") && !c.identifier() {
        return false;
    }
    if !c.word("ON") {
        return false;
    }
    let object = [
        "TABLE",
        "COLUMN",
        "AGGREGATE",
        "DATABASE",
        "DOMAIN",
        "EVENT",
        "FOREIGN",
        "FUNCTION",
        "LARGE",
        "MATERIALIZED",
        "PROCEDURAL",
        "PROCEDURE",
        "PUBLICATION",
        "ROLE",
        "ROUTINE",
        "SCHEMA",
        "SEQUENCE",
        "SUBSCRIPTION",
        "TABLESPACE",
        "TYPE",
        "VIEW",
    ]
    .iter()
    .find(|word| c.word(word))
    .copied();
    let Some(object) = object else {
        return false;
    };
    object_name(c, object) && c.word("IS") && (c.string() || c.word("NULL"))
}

fn object_name(c: &mut Cursor<'_>, object: &str) -> bool {
    if object == "LARGE" {
        if !c.word("OBJECT") || !c.number() {
            return false;
        }
    } else if object == "EVENT" {
        if !c.word("TRIGGER") || !c.identifier() {
            return false;
        }
    } else if object == "FOREIGN" {
        if !c.word("TABLE") || !c.name() {
            return false;
        }
    } else if object == "MATERIALIZED" {
        if !c.word("VIEW") || !c.name() {
            return false;
        }
    } else if object == "PROCEDURAL" {
        if !c.word("LANGUAGE") || !c.identifier() {
            return false;
        }
    } else if object == "COLUMN" {
        if !signatures::qualified_column(c) {
            return false;
        }
    } else if object == "FUNCTION"
        || object == "PROCEDURE"
        || object == "ROUTINE"
        || object == "AGGREGATE"
    {
        if !c.function_name() || !signatures::signature(c, object == "AGGREGATE") {
            return false;
        }
    } else if matches!(
        object,
        "DATABASE" | "PUBLICATION" | "ROLE" | "SCHEMA" | "SUBSCRIPTION" | "TABLESPACE"
    ) {
        if !c.identifier() {
            return false;
        }
    } else if !c.name() {
        return false;
    }
    true
}

use super::support::expect_err;

#[test]
fn schema_catalog_path_is_required() {
    expect_err("patterns: {}", "option schemaCatalogPath: required");
    expect_err(
        "schemaCatalogPath: '   '\n",
        "option schemaCatalogPath: required",
    );
}

#[test]
fn unknown_pattern_kind_and_invalid_regex_fail() {
    expect_err(
        "schemaCatalogPath: schema.json\npatterns:\n  sequence: '^[a-z]+$'\n",
        "option patterns.sequence: unknown pattern kind",
    );
    expect_err(
        "schemaCatalogPath: schema.json\npatterns:\n  table: '('\n",
        "option patterns.table: invalid regex:",
    );
}

#[test]
fn table_placeholder_is_rejected_outside_index_trigger_and_unique_index() {
    for kind in [
        "table",
        "column",
        "function",
        "triggerFunction",
        "view",
        "materializedView",
        "enum",
    ] {
        expect_err(
            &format!("schemaCatalogPath: schema.json\npatterns:\n  {kind}: '^x_{{table}}$'\n"),
            &format!(
                "option patterns.{kind}: {{table}} is only allowed in index, uniqueIndex and trigger"
            ),
        );
    }
}

#[test]
fn table_placeholder_shape_errors() {
    let prefix = "schemaCatalogPath: schema.json\npatterns:\n  index: ";
    expect_err(
        &format!("{prefix}'^idx_{{table}}_{{table}}$'\n"),
        "option patterns.index: {table} may appear only once",
    );
    expect_err(
        &format!("{prefix}'idx_{{table}}$'\n"),
        "option patterns.index: a pattern with {table} must start with ^ and end with $",
    );
    expect_err(
        &format!("{prefix}'^idx_{{table}}'\n"),
        "option patterns.index: a pattern with {table} must start with ^ and end with $",
    );
    expect_err(
        &format!("{prefix}'^idx_{{table}}\\$'\n"),
        "option patterns.index: a pattern with {table} must start with ^ and end with $",
    );
    expect_err(
        &format!("{prefix}'^(idx_{{table}})$'\n"),
        "option patterns.index: {table} must not be inside a group, a character class or an alternation",
    );
    expect_err(
        &format!("{prefix}'^[{{table}}]$'\n"),
        "option patterns.index: {table} must not be inside a group, a character class or an alternation",
    );
    expect_err(
        &format!("{prefix}'^idx_{{table}}$|^plain$'\n"),
        "option patterns.index: {table} must not be inside a group, a character class or an alternation",
    );
    expect_err(
        &format!("{prefix}'^a{{1{{table}}$'\n"),
        "option patterns.index: invalid regex:",
    );
    expect_err(
        &format!("{prefix}'^idx_{{table}}*$'\n"),
        "option patterns.index: invalid regex:",
    );
}

#[test]
fn numeric_and_plural_options_fail_closed() {
    expect_err(
        "schemaCatalogPath: schema.json\ntableMinWords: 0\n",
        "option tableMinWords: must be at least 1",
    );
    expect_err(
        "schemaCatalogPath: schema.json\nabbreviations:\n  minLetters: 0\n",
        "option abbreviations.minLetters: must be at least 1",
    );
    expect_err(
        "schemaCatalogPath: schema.json\nplural:\n  objects: []\n",
        "option plural.objects: must not be empty",
    );
    expect_err(
        "schemaCatalogPath: schema.json\nplural:\n  enabled: true\n  objects: [view]\n",
        "option plural.objects: unknown value view",
    );
    expect_err(
        "schemaCatalogPath: schema.json\nplural:\n  irregularPlurals: {'': people}\n",
        "option plural.irregularPlurals: empty key",
    );
    expect_err(
        "schemaCatalogPath: schema.json\nplural:\n  irregularPlurals: {person: ''}\n",
        "option plural.irregularPlurals: empty value",
    );
    expect_err(
        "schemaCatalogPath: schema.json\nplural:\n  enabled: false\n  ignorePatterns: ['[']\n",
        "option plural.ignorePatterns: invalid regex:",
    );
}

#[test]
fn token_spelling_underscore_and_allow_options_fail_closed() {
    expect_err(
        "schemaCatalogPath: schema.json\ndeniedTokens:\n  - token: ''\n    replacement: configuration\n",
        "option deniedTokens: empty token",
    );
    expect_err(
        "schemaCatalogPath: schema.json\ndeniedTokens:\n  - {token: cfg, replacement: configuration}\n  - {token: CFG, replacement: config}\n",
        "option deniedTokens: duplicate token CFG",
    );
    expect_err(
        "schemaCatalogPath: schema.json\nspelling:\n  '': acknowledgment\n",
        "option spelling: empty key",
    );
    expect_err(
        "schemaCatalogPath: schema.json\nspelling:\n  acknowledgement: acknowledgement\n",
        "option spelling: key \"acknowledgement\" equals its value",
    );
    expect_err(
        "schemaCatalogPath: schema.json\nspelling:\n  ID: id\n",
        "option spelling: key \"ID\" equals its value",
    );
    expect_err(
        "schemaCatalogPath: schema.json\ndoubleUnderscore:\n  allowPattern: '['\n",
        "option doubleUnderscore.allowPattern: invalid regex:",
    );
    expect_err(
        "schemaCatalogPath: schema.json\nallow:\n  - {object: table:accounts, reason: '   '}\n",
        "option allow: entry table:accounts needs a reason",
    );
    expect_err(
        "schemaCatalogPath: schema.json\nallow:\n  - {object: nope, reason: kept}\n",
        "option allow: invalid object ref nope",
    );
    expect_err(
        "schemaCatalogPath: schema.json\nallow:\n  - {object: 'table:accounts', reason: kept}\n  - {object: 'table:accounts', reason: again}\n",
        "option allow: duplicate entry table:accounts",
    );
}

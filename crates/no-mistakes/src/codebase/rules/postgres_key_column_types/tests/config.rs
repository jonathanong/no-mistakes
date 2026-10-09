use super::support::{expect_err, PATH};

#[test]
fn validates_required_types_duplicates_flags_and_constraint_allow_entries() {
    expect_err("", "schemaCatalogPath: required");
    expect_err(
        &format!("{PATH}allowedTypes: []\n"),
        "allowedTypes: required and nonempty",
    );
    expect_err(
        &format!("{PATH}allowedTypes: [uuid, ' UUID ']\n"),
        "duplicate entry UUID",
    );
    expect_err(
        &format!("{PATH}allowedTypes: [uuid, ' ']\n"),
        "entries must be nonempty",
    );
    expect_err(
        &format!("{PATH}allowedTypes: [uuid]\ncheckPrimaryKeys: false\ncheckForeignKeys: false\n"),
        "at least one must be true",
    );
    expect_err(&format!("{PATH}allowedTypes: [uuid]\nallow:\n  - object: 'column:orders.id'\n    reason: not a constraint\n"), "expected a constraint object ref");
    expect_err(&format!("{PATH}allowedTypes: [uuid]\nallow:\n  - object: 'constraint:'\n    reason: invalid ref\n"), "invalid object ref");
    expect_err(&format!("{PATH}allowedTypes: [uuid]\nallow:\n  - object: 'constraint:orders.orders_pkey'\n    reason: ' '\n"), "needs a reason");
    expect_err(&format!("{PATH}allowedTypes: [uuid]\nallow:\n  - object: 'constraint:orders.orders_pkey'\n    reason: one\n  - object: 'constraint:orders.orders_pkey'\n    reason: two\n"), "duplicate entry");
}

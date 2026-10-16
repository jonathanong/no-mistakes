-- Other-dialect AST shapes exercise conservative shared constraint projections.
CREATE TABLE indexed_values (
    base_value integer,
    UNIQUE KEY expression_key ((base_value + 1)),
    INDEX helper_index (base_value)
);

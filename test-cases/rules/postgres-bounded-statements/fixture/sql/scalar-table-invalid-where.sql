-- TABLE does not accept a WHERE clause.
CREATE VIEW v AS SELECT (TABLE helper WHERE false);

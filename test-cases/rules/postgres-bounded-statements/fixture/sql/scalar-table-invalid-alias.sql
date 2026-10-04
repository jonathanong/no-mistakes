-- TABLE does not accept a SELECT-style source alias.
CREATE VIEW v AS SELECT (TABLE helper AS h);

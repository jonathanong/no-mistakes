SELECT o.id FROM orders o JOIN qualified_source v ON v.id = COALESCE(o.account_id, o.id);
SELECT o.id FROM orders o JOIN wildcard_alias v ON v.id = COALESCE(o.account_id, o.id);

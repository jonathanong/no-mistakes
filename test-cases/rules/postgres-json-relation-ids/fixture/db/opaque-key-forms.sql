-- Dynamic document keys and opaque row projections are deliberately outside this rule.
INSERT INTO orders (details) SELECT jsonb_build_object(key_name, value) FROM staging;
INSERT INTO orders TABLE staged_orders;
INSERT INTO orders SELECT * FROM staged_orders s JOIN staged_accounts a USING (account_id);
UPDATE orders SET (details, alternate_id) = (SELECT document, id FROM staging LIMIT 1);

-- CASE and COALESCE choose a top-level JSON value, rather than nesting a key.
UPDATE orders SET details = CASE WHEN $1 THEN jsonb_build_object('accountId', $2) ELSE '{}'::jsonb END;
INSERT INTO orders (details) VALUES (COALESCE(NULL, jsonb_build_object('accountId', $1)));

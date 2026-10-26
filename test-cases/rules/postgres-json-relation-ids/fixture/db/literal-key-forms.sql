-- These are literal keys even when PostgreSQL spells the string or JSONB cast explicitly.
INSERT INTO orders (details) VALUES (jsonb_build_object(E'accountId', $1));
INSERT INTO orders (details) VALUES (jsonb_build_object(U&'accountId', $1));
INSERT INTO orders (details) VALUES (CAST(jsonb_build_object('accountId', $1) AS jsonb));
INSERT INTO orders (account_id, details) VALUES ($1, CAST(jsonb_build_object('accountId', $1) AS jsonb));

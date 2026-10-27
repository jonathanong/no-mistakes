-- Missing catalog ordinals cannot prove that a positional value writes account_id.
INSERT INTO orders VALUES (jsonb_build_object('accountId', $1));
-- Explicitly naming the FK remains sufficient without ordinal metadata.
INSERT INTO orders (account_id, details) VALUES ($1, jsonb_build_object('accountId', $1));

-- A catalog is optional for literal-key checks; missing column metadata must not hide the key.
INSERT INTO events VALUES (jsonb_build_object('accountId', $1));

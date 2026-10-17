-- Preserve forms accepted by PostgreSqlDialect, including alternate FROM and MERGE wildcards.
UPDATE users u FROM accounts a SET active = true WHERE u.id = a.user_id
RETURNING u.*, row_to_json(record => u.*);
UPDATE users u SET active = true FROM accounts a WHERE u.id = a.user_id RETURNING u.*;
INSERT INTO public.users DEFAULT VALUES;
INSERT INTO public.users VALUES (1, true);
MERGE INTO users u USING accounts a ON u.id = a.id
WHEN MATCHED THEN UPDATE SET *
WHEN NOT MATCHED THEN INSERT *;
MERGE INTO users u USING accounts a ON u.id = a.id
WHEN MATCHED THEN DELETE
WHEN NOT MATCHED THEN INSERT (id, active) VALUES (a.id, true);

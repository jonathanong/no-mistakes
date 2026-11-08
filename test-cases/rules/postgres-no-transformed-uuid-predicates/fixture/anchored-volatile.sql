SELECT * FROM accounts WHERE id = gen_random_uuid() AND fn(token) = $1;
SELECT * FROM accounts WHERE id = (gen_random_uuid()::text)::uuid AND fn(token) = $1;
SELECT * FROM accounts WHERE id = COALESCE(gen_random_uuid(), $1) AND fn(token) = $2;
SELECT * FROM accounts WHERE id >= uuidv7() AND fn(token) = $1;
SELECT * FROM accounts WHERE id = ANY(ARRAY[gen_random_uuid()]) AND fn(token) = $1;
SELECT * FROM accounts WHERE id = $1::uuid AND fn(token) = $2;
SELECT * FROM accounts WHERE id = COALESCE($1::uuid, $2::uuid) AND fn(token) = $3;
SELECT * FROM accounts WHERE fn(token) = $1;
SELECT * FROM accounts WHERE id = pg_catalog.gen_random_uuid() AND fn(token) = $1;

-- Aggregate values and parameter/text-side functions leave UUID lookups bare.
SELECT id FROM users GROUP BY id HAVING COUNT(DISTINCT id) >= 2;
SELECT id FROM users GROUP BY id HAVING MIN(id) >= $1;
SELECT id FROM users GROUP BY id HAVING MAX(id) < $1;
SELECT count(*) FROM users HAVING COUNT(id::text) >= 2;
SELECT count(*) FROM users HAVING cardinality(ARRAY_AGG(id)) >= 2;
SELECT count(*) FROM users HAVING lower(MIN(id)::text) = $1;
SELECT id FROM users WHERE id >= fn_min_child_uuidv7($1);
SELECT id FROM users WHERE fn_min_child_uuidv7($1) <= id;
SELECT id FROM users WHERE id >= fn_min_child_uuidv7(email);
SELECT u.id FROM users u JOIN sessions s ON u.id >= fn_min_child_uuidv7($1);
SELECT id FROM users WHERE uuid_extract_timestamp(id) >= $1;
SELECT id FROM users WHERE id::text = $1;
SELECT id FROM users WHERE id >= fn_min_child_uuidv7(external_ref);
SELECT count(*) FROM users HAVING count(id) FILTER (WHERE id::text = $1) >= 2;
SELECT u.id FROM users u JOIN sessions s ON uuid_extract_timestamp(u.id) >= $1;
SELECT id FROM users GROUP BY id HAVING uuid_extract_timestamp(id) >= $1;

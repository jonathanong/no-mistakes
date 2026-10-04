SELECT id FROM orders WHERE $1::boolean IS NOT NULL AND id > $2 ORDER BY id LIMIT $3;
SELECT id FROM orders WHERE $1::boolean IS NOT NULL AND id > $2 AND deleted_at IS NULL ORDER BY id LIMIT $3;
SELECT id FROM orders WHERE CASE WHEN $1::boolean THEN (orders.*, NULL) IS NULL ELSE FALSE END AND id > $2 ORDER BY id LIMIT $3; -- Qualified wildcards keep this predicate row-dependent.
SELECT id FROM orders WHERE ($1::guard_options).enabled IS TRUE AND id > $2 ORDER BY id LIMIT $3; -- The dotted field name is a label; this is a bind-only guard.
SELECT id FROM orders WHERE "sql_placeholder_1" = $1 AND id > $2 ORDER BY id LIMIT $3; -- Quoted placeholder-like names are real columns.
SELECT id FROM orders WHERE sql_placeholder_1 = $1 AND id > $2 ORDER BY id LIMIT $3; -- In standalone SQL, the recovery prefix is an ordinary identifier.

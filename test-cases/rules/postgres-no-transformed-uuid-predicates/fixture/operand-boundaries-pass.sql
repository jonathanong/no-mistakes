-- Wrappers consume boolean results or CASE results; these UUID comparison operands stay bare.
SELECT 1 FROM accounts WHERE CAST(id = $1 AS boolean);
SELECT 1 FROM accounts WHERE COALESCE(id = $1, false);
SELECT 1 FROM accounts WHERE CAST(NOT (id = $1) AS boolean);
SELECT 1 FROM accounts WHERE COALESCE(id IS NULL, false);
SELECT 1 FROM accounts WHERE COALESCE((id = $1) IS TRUE, false);
SELECT 1 FROM accounts WHERE CAST(id IS DISTINCT FROM $1 AS boolean);
SELECT 1 FROM accounts WHERE CAST(id BETWEEN $1 AND $2 AS boolean);
SELECT 1 FROM accounts WHERE CAST(id IN ($1, $2) AS boolean);
SELECT 1 FROM accounts WHERE CAST(id = ANY($1::uuid[]) AS boolean);
SELECT 1 FROM accounts WHERE CAST(CASE WHEN id = $1 THEN email ELSE email END AS text) = $2;
SELECT 1 FROM accounts WHERE CAST(CASE id WHEN $1 THEN email ELSE email END AS text) = $2;
SELECT 1 FROM accounts WHERE COALESCE(id = $1 AND id <> $2, false);
SELECT 1 FROM accounts WHERE COALESCE(id <= $1 OR id > $2, false);
SELECT 1 FROM accounts WHERE COALESCE((id < $1) IS NOT TRUE, false);
SELECT 1 FROM accounts WHERE COALESCE(ARRAY[id] && ARRAY[$1::uuid], false);
SELECT 1 FROM accounts WHERE lower(CASE WHEN id = $1 THEN email ELSE email END) = $2;
SELECT 1 FROM accounts WHERE COALESCE(id IN (SELECT id FROM orders), false);

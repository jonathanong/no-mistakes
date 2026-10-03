-- NOT reverses the cursor side, but still walks a key range.
SELECT * FROM accounts WHERE NOT (id <= $1) ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE NOT (id >= $1) ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE NOT NOT (id > $1) ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE NOT ($1 >= id) ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE NOT ((id, created_at) <= ($1, $2)) ORDER BY id, created_at LIMIT $3;
-- Two opposite bounds form a selective window, including a reversed NOT cursor.
SELECT * FROM accounts WHERE NOT (id <= $1) AND id < $2 ORDER BY id LIMIT $3;
SELECT * FROM accounts WHERE NOT (id >= $1) AND id > $2 ORDER BY id LIMIT $3;
-- Arbitrary negated predicates are not proven cursors.
SELECT * FROM accounts WHERE NOT (id = $1) ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE NOT ($1 IS NULL OR id > $1) ORDER BY id LIMIT $2;

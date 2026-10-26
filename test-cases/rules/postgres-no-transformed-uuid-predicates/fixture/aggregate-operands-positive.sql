-- Aggregate FILTER and nested query predicates still constrain rows, as do ordinary scalar calls.
SELECT email FROM accounts GROUP BY email HAVING COUNT(id) FILTER (WHERE id::text = $1) > 1;
SELECT email FROM accounts GROUP BY email HAVING COUNT(id) FILTER (WHERE fn(id) > $1) > 1;
SELECT email FROM accounts GROUP BY email HAVING fn(id) >= $1;
SELECT email FROM accounts GROUP BY email HAVING app.count(id) > 1;
SELECT email FROM accounts GROUP BY email HAVING app.min(id) = $1;
SELECT 1 FROM accounts WHERE fn(id) >= $1;
SELECT 1 FROM accounts WHERE id::text = $1;
SELECT 1 FROM accounts WHERE id >= fn(token);
SELECT 1 FROM accounts WHERE fn(token) <= id;
SELECT id FROM accounts GROUP BY id, token HAVING COALESCE(MIN(id), token) = $1;
SELECT 1 FROM text_ids WHERE fn(id) >= $1;
SELECT 1 FROM accounts WHERE fn(email) >= $1;
SELECT email FROM accounts GROUP BY email HAVING "MIN"(id) = $1;
SELECT email FROM accounts GROUP BY email HAVING COUNT((SELECT 1 FROM accounts a WHERE a.id::text = $1)) > 1;
SELECT email FROM accounts GROUP BY email HAVING COALESCE(COUNT(id) FILTER (WHERE id::text = $1), 0) > 1;
SELECT email FROM accounts GROUP BY email HAVING COALESCE(COUNT(id) FILTER (WHERE fn(id) = $1), 0) > 1;

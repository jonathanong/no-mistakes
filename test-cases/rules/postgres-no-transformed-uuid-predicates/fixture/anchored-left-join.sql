SELECT * FROM accounts e LEFT JOIN accounts a ON a.id = e.token LEFT JOIN accounts b ON b.id = e.token WHERE num_nonnulls(a.id, b.id) = 1;
SELECT * FROM accounts e LEFT JOIN accounts a ON a.id::text = e.email WHERE num_nonnulls(a.id, e.token) = 1;
SELECT * FROM accounts a JOIN accounts b ON b.id = $1 WHERE num_nonnulls(a.id, b.id) = 1;

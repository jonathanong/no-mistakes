-- Composite key candidates are finite only when the source is bounded.
DELETE FROM order_lines WHERE (order_id, line_no) IN (SELECT order_id, line_no FROM order_lines LIMIT $1);
DELETE FROM order_lines WHERE (order_id, line_no) IN (SELECT * FROM unnest($1::uuid[], $2::int[]));
DELETE FROM order_lines WHERE (order_id, line_no) IN (SELECT order_id, line_no FROM order_lines);
-- The array's input, rather than its aggregate output row, must be bounded.
WITH batch AS (SELECT id FROM accounts LIMIT $1) DELETE FROM accounts WHERE id = ANY(ARRAY(SELECT id FROM batch));
WITH batch AS (SELECT id FROM accounts LIMIT $1) DELETE FROM accounts WHERE id = ANY((SELECT array_agg(id) FROM batch)::uuid[]);
DELETE FROM accounts WHERE id = ANY((SELECT array_agg(id) FROM accounts)::uuid[]);
-- Independent lateral lookup already has a global bound; a per-driver cap does not bound drivers.
SELECT a.id FROM LATERAL (SELECT id FROM accounts ORDER BY id LIMIT 1) latest JOIN accounts a ON a.id = latest.id;
SELECT a.id FROM orders o CROSS JOIN LATERAL (SELECT id FROM accounts WHERE id = o.account_id LIMIT 1) latest JOIN accounts a ON a.id = latest.id;
-- A guaranteed caller row preserves a limit contract. Empty/NULL/data-derived counts do not.
WITH cursor_cte AS (SELECT $1::int AS page_size) SELECT id FROM accounts LIMIT (SELECT page_size FROM cursor_cte);
WITH cursor_cte AS (SELECT $1::int AS page_size WHERE false) SELECT id FROM accounts LIMIT (SELECT page_size FROM cursor_cte);
WITH cursor_cte AS (SELECT NULL::int AS page_size) SELECT id FROM accounts LIMIT (SELECT page_size FROM cursor_cte);
WITH cursor_cte AS (SELECT line_no AS page_size FROM order_lines LIMIT 1) SELECT id FROM accounts LIMIT (SELECT page_size FROM cursor_cte);
-- Each OR alternative must complete a real key, never pool columns across branches.
SELECT id FROM accounts WHERE id = $1 OR email = $2;
SELECT id FROM accounts WHERE id = $1 OR name = $2;
SELECT * FROM order_lines WHERE order_id = $1 OR line_no = $2;
-- One row per recursive step does not bound total depth; UNION ALL may cycle forever.
WITH RECURSIVE chain AS (SELECT id, account_id FROM orders WHERE id = $1 UNION ALL SELECT p.id, p.account_id FROM orders p JOIN chain c ON p.id = c.account_id) SELECT * FROM chain;

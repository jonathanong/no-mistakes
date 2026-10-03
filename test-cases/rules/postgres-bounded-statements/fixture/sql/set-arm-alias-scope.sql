-- The second arm's local a cannot hide the first arm's outer a.
DELETE FROM accounts a WHERE a.id IN (SELECT a.id UNION ALL SELECT a.id FROM orders a WHERE a.id = $1);
-- Reversing the arms keeps the same correlation.
DELETE FROM accounts a WHERE a.id IN (SELECT a.id FROM orders a WHERE a.id = $1 UNION ALL SELECT a.id);
-- Independently pinned sources stay bounded when each arm owns its aliases.
DELETE FROM accounts a WHERE a.id IN (SELECT a.id FROM orders a WHERE a.id = $1 UNION ALL SELECT a.id FROM orders a WHERE a.id = $2);
-- Combined ORDER BY output names must not hide a bare outer reference in an arm.
DELETE FROM accounts a WHERE a.id IN (SELECT id UNION ALL SELECT code FROM currencies ORDER BY id LIMIT 1);
-- An arm's GROUP BY label does not hide the column read by that arm's projection.
DELETE FROM accounts a WHERE a.id IN (SELECT id GROUP BY id UNION ALL SELECT code FROM currencies LIMIT 1);
-- Grouping expressions belong to their local arm, even when they are not output labels.
DELETE FROM accounts a WHERE a.id IN (SELECT lower(code) FROM currencies GROUP BY lower(code) UNION ALL SELECT code FROM currencies LIMIT 1);
-- GROUP BY ALL has no explicit output labels to register in an arm.
DELETE FROM accounts a WHERE a.id IN (SELECT code FROM currencies GROUP BY ALL UNION ALL SELECT code FROM currencies LIMIT 1);
-- The same ALL grouping remains local in an ordinary query.
DELETE FROM accounts a WHERE a.id IN (SELECT code FROM currencies GROUP BY ALL LIMIT 1);

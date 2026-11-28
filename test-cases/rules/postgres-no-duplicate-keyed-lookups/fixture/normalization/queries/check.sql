SELECT
    (SELECT a.rank AS sort_key FROM accounts a
     WHERE a.id = o.account_id ORDER BY sort_key LIMIT 1),
    (SELECT b.priority AS sort_key FROM accounts b
     WHERE b.id = o.account_id ORDER BY sort_key LIMIT 1)
FROM orders o;

SELECT
    (SELECT a.rank FROM accounts a
     WHERE a.id = o.account_id ORDER BY 1 LIMIT 1),
    (SELECT b.priority FROM accounts b
     WHERE b.id = o.account_id ORDER BY 1 LIMIT 1)
FROM orders o;

SELECT
    (SELECT a.name FROM accounts a
     WHERE a.id = -CAST(o.account_id IS NULL AS integer) + 1 AND pg_catalog.abs(1) = 1
       AND a.archived_at IS NULL AND a.email IS NOT NULL
       AND a.active IS TRUE AND a.locked IS FALSE
       AND a.rank IS DISTINCT FROM o.rank AND a.team_id IS NOT DISTINCT FROM o.team_id
       AND a.tier IN ('gold', 'silver') AND a.created_on IS NOT NULL
       AND a.name = 'unknown'::text
     ORDER BY a.priority DESC NULLS LAST LIMIT 1),
    (SELECT b.tier FROM accounts b
     WHERE -CAST(o.account_id IS NULL AS integer) + 1 = b.id AND pg_catalog.abs(1) = 1
       AND b.archived_at IS NULL AND b.email IS NOT NULL
       AND b.active IS TRUE AND b.locked IS FALSE
       AND b.rank IS DISTINCT FROM o.rank AND b.team_id IS NOT DISTINCT FROM o.team_id
       AND b.tier IN ('silver', 'gold') AND b.created_on IS NOT NULL
       AND b.name = CAST('unknown' AS text)
     ORDER BY b.priority DESC NULLS LAST LIMIT 1),
    (SELECT c.tier FROM accounts c
     WHERE -CAST(o.account_id IS NULL AS integer) + 1 = c.id AND pg_catalog.abs(1) = 1
       AND c.archived_at IS NULL AND c.email IS NOT NULL
       AND c.active IS TRUE AND c.locked IS FALSE
       AND c.rank IS DISTINCT FROM o.rank AND c.team_id IS NOT DISTINCT FROM o.team_id
       AND c.tier IN ('silver', 'gold') AND c.created_on IS NOT NULL
       AND c.name = 'unknown'::text
     ORDER BY c.priority DESC NULLS FIRST LIMIT 1)
FROM orders o;

SELECT o.id
FROM orders o
LEFT JOIN accounts a ON a.created_on = DATE '2020-01-01'
LEFT JOIN accounts b ON b.created_on = DATE '2020-01-01';

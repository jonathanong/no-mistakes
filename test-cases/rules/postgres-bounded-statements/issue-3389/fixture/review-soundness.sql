WITH quotas(n) AS (VALUES (1))
SELECT *
FROM (
  SELECT p.n FROM quotas() p
) q
CROSS JOIN LATERAL (
  SELECT reservation_id FROM consumptions LIMIT q.n
) r;
SELECT reservation_id
FROM budgets
CROSS JOIN LATERAL (
  SELECT reservation_id FROM consumptions
  INTERSECT
  SELECT actor_id FROM quotas
  LIMIT budgets.cap
) AS capped
WHERE budgets.actor_id = $1;
SELECT reservation_id
FROM budgets
CROSS JOIN LATERAL (
  SELECT reservation_id FROM consumptions
  EXCEPT
  SELECT actor_id FROM quotas
  LIMIT budgets.cap
) AS capped
WHERE budgets.actor_id = $1;
SELECT reservation_id
FROM budgets
CROSS JOIN LATERAL (
  SELECT reservation_id FROM consumptions
  UNION
  SELECT actor_id FROM quotas
  LIMIT budgets.cap
) AS capped
WHERE budgets.actor_id = $1;
SELECT reservation_id
FROM budgets
CROSS JOIN LATERAL (
  SELECT reservation_id FROM consumptions
  UNION ALL
  SELECT actor_id FROM quotas
  LIMIT budgets.cap
) AS capped
WHERE budgets.actor_id = $1;
SELECT committed_at
FROM caps
CROSS JOIN LATERAL (
  SELECT committed_at FROM consumptions LIMIT caps.ctid
) latest;
SELECT committed_at
FROM budgets
CROSS JOIN LATERAL (
  SELECT committed_at FROM consumptions LIMIT budgets.ctid
) latest
WHERE budgets.actor_id = $1;

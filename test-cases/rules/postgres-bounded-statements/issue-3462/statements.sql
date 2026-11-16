SELECT committed_at
FROM archive.budgets
CROSS JOIN LATERAL (
  SELECT committed_at FROM consumptions LIMIT archive.budgets.cap
) latest;
SELECT committed_at
FROM budgets
CROSS JOIN LATERAL (
  SELECT committed_at FROM consumptions LIMIT budgets.cap
) latest;
SELECT committed_at
FROM safe
CROSS JOIN LATERAL (
  SELECT committed_at FROM consumptions LIMIT safe.cap
) latest
WHERE safe.actor_id = $1;

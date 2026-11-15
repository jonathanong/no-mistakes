WITH windows(capacity_limit) AS (VALUES (2))
SELECT windows.capacity_limit, probe.used, probe.retry_at
FROM windows
CROSS JOIN LATERAL (
  SELECT count(*) AS used, min(committed_at) AS retry_at
  FROM (
    SELECT committed_at
    FROM consumptions
    WHERE actor_id = $1
    ORDER BY committed_at DESC, reservation_id DESC
    LIMIT GREATEST(windows.capacity_limit, 0)
  ) latest
) probe;
SELECT committed_at
FROM consumptions
WHERE actor_id = $1
ORDER BY committed_at DESC, reservation_id DESC
LIMIT $2;
WITH windows(capacity_limit) AS (VALUES (2))
SELECT committed_at
FROM windows
CROSS JOIN LATERAL (
  SELECT committed_at
  FROM consumptions
  WHERE actor_id = $1
  ORDER BY committed_at DESC, reservation_id DESC
  LIMIT windows.capacity_limit
) latest;
WITH windows(capacity_limit) AS (VALUES (2))
SELECT committed_at
FROM windows
CROSS JOIN (
  SELECT committed_at
  FROM consumptions
  WHERE actor_id = $1
  LIMIT windows.capacity_limit
) latest;
SELECT committed_at
FROM consumptions
WHERE actor_id = $1
LIMIT (SELECT count(*) FROM consumptions);
SELECT committed_at
FROM consumptions
WHERE actor_id = $1
LIMIT NULL;
WITH windows(capacity_limit) AS (VALUES (2))
SELECT committed_at
FROM windows
CROSS JOIN LATERAL (
  SELECT committed_at
  FROM consumptions
  WHERE actor_id = $1
  LIMIT GREATEST(windows.capacity_limit, NULL)
) latest;
SELECT committed_at
FROM consumptions
WHERE actor_id = $1
ORDER BY committed_at DESC
LIMIT consumptions.actor_id;
WITH windows(capacity_limit) AS (VALUES (2))
SELECT committed_at
FROM windows
CROSS JOIN LATERAL (
  SELECT committed_at
  FROM consumptions
  WHERE actor_id = $1
  LIMIT missing.capacity_limit
) latest;
SELECT committed_at
FROM quotas
CROSS JOIN LATERAL (
  SELECT committed_at
  FROM consumptions
  WHERE actor_id = $1
  LIMIT quotas.cap
) latest
WHERE quotas.actor_id = $1;
WITH missing(cap) AS (VALUES (NULL::int))
SELECT committed_at
FROM missing
CROSS JOIN LATERAL (
  SELECT committed_at
  FROM consumptions
  WHERE actor_id = $1
  LIMIT missing.cap
) latest;
WITH windows(capacity_limit) AS (VALUES (2))
SELECT committed_at
FROM windows
LEFT JOIN (SELECT 2 AS capacity_limit) extra ON false
CROSS JOIN LATERAL (
  SELECT committed_at
  FROM consumptions
  WHERE actor_id = $1
  LIMIT extra.capacity_limit
) latest;
WITH missing(cap) AS (VALUES (NULL::int))
SELECT committed_at
FROM missing
CROSS JOIN LATERAL (
  SELECT committed_at
  FROM consumptions
  WHERE actor_id = $1
  LIMIT GREATEST(missing.cap, 0)
) latest;
WITH missing(cap) AS (VALUES (NULL::int))
SELECT committed_at
FROM missing
CROSS JOIN LATERAL (
  SELECT committed_at
  FROM consumptions
  WHERE actor_id = $1
  LIMIT COALESCE(missing.cap, 0)
) latest;
SELECT committed_at
FROM consumptions
CROSS JOIN LATERAL (
  SELECT 1
  LIMIT cap
) marker
WHERE actor_id = $1;
SELECT committed_at
FROM budgets
CROSS JOIN LATERAL (
  SELECT committed_at
  FROM consumptions
  WHERE actor_id = $1
  LIMIT budgets.cap
) latest
WHERE budgets.actor_id = $1;
SELECT budgets.actor_id
FROM budgets
CROSS JOIN LATERAL (
  SELECT generate_series(1, 0)
  FROM consumptions
  LIMIT budgets.cap
) AS series
WHERE budgets.actor_id = $1;

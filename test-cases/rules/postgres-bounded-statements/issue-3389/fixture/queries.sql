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

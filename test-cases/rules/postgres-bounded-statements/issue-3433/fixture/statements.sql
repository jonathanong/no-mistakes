-- pass parameter-returning-key
WITH requested AS (
  SELECT predecessor_post_id FROM jsonb_to_recordset($1::jsonb) AS x(predecessor_post_id uuid)
),
inserted AS (
  INSERT INTO repro_successions (predecessor_post_id, predecessor_archived_at)
  SELECT r.predecessor_post_id, CURRENT_TIMESTAMP
  FROM requested r JOIN repro_posts p ON p.id = r.predecessor_post_id
  WHERE p.archived_at IS NULL
  ON CONFLICT (predecessor_post_id, predecessor_archived_at) DO NOTHING
  RETURNING predecessor_post_id, predecessor_archived_at
)
UPDATE repro_posts p
SET archived_at = i.predecessor_archived_at
FROM inserted i
WHERE p.id = i.predecessor_post_id AND p.archived_at IS NULL;

-- pass any-array
UPDATE repro_posts SET archived_at = CURRENT_TIMESTAMP WHERE id = ANY($1::uuid[]);

-- report non-key-returning
WITH requested AS (
  SELECT predecessor_post_id FROM jsonb_to_recordset($1::jsonb) AS x(predecessor_post_id uuid)
),
inserted AS (
  INSERT INTO repro_successions (predecessor_post_id, predecessor_archived_at)
  SELECT r.predecessor_post_id, CURRENT_TIMESTAMP
  FROM requested r JOIN repro_posts p ON p.id = r.predecessor_post_id
  WHERE p.archived_at IS NULL
  ON CONFLICT (predecessor_post_id, predecessor_archived_at) DO NOTHING
  RETURNING predecessor_archived_at
)
UPDATE repro_posts p
SET archived_at = i.predecessor_archived_at
FROM inserted i
WHERE p.archived_at = i.predecessor_archived_at;

-- report unkeyed-source
WITH requested AS (
  SELECT predecessor_post_id FROM jsonb_to_recordset($1::jsonb) AS x(predecessor_post_id uuid)
),
inserted AS (
  INSERT INTO repro_successions (predecessor_post_id, predecessor_archived_at)
  SELECT p.id, CURRENT_TIMESTAMP
  FROM requested r JOIN repro_posts p ON p.archived_at IS NULL
  ON CONFLICT (predecessor_post_id, predecessor_archived_at) DO NOTHING
  RETURNING predecessor_post_id
)
UPDATE repro_posts p
SET archived_at = CURRENT_TIMESTAMP
FROM inserted i
WHERE p.id = i.predecessor_post_id;

-- report quoted-insert
WITH requested AS (
  SELECT predecessor_post_id FROM jsonb_to_recordset($1::jsonb) AS x(predecessor_post_id uuid)
),
inserted AS (
  INSERT INTO repro_successions (predecessor_post_id, predecessor_archived_at)
  SELECT r.predecessor_post_id, "current_timestamp"(id)
  FROM requested r JOIN repro_posts p ON p.id = r.predecessor_post_id
  WHERE p.archived_at IS NULL
  ON CONFLICT (predecessor_post_id, predecessor_archived_at) DO NOTHING
  RETURNING predecessor_post_id, predecessor_archived_at
)
UPDATE repro_posts p
SET archived_at = i.predecessor_archived_at
FROM inserted i
WHERE p.id = i.predecessor_post_id AND p.archived_at IS NULL;

-- report qualified-insert
WITH requested AS (
  SELECT predecessor_post_id FROM jsonb_to_recordset($1::jsonb) AS x(predecessor_post_id uuid)
),
inserted AS (
  INSERT INTO repro_successions (predecessor_post_id, predecessor_archived_at)
  SELECT r.predecessor_post_id, pg_catalog.current_timestamp()
  FROM requested r JOIN repro_posts p ON p.id = r.predecessor_post_id
  WHERE p.archived_at IS NULL
  ON CONFLICT (predecessor_post_id, predecessor_archived_at) DO NOTHING
  RETURNING predecessor_post_id, predecessor_archived_at
)
UPDATE repro_posts p
SET archived_at = i.predecessor_archived_at
FROM inserted i
WHERE p.id = i.predecessor_post_id AND p.archived_at IS NULL;

-- report column-recordset
WITH requested AS (
  SELECT x.predecessor_post_id
  FROM repro_batches b
  CROSS JOIN LATERAL jsonb_to_recordset(b.payload) AS x(predecessor_post_id uuid)
),
inserted AS (
  INSERT INTO repro_successions (predecessor_post_id, predecessor_archived_at)
  SELECT r.predecessor_post_id, CURRENT_TIMESTAMP
  FROM requested r JOIN repro_posts p ON p.id = r.predecessor_post_id
  WHERE p.archived_at IS NULL
  ON CONFLICT (predecessor_post_id, predecessor_archived_at) DO NOTHING
  RETURNING predecessor_post_id, predecessor_archived_at
)
UPDATE repro_posts p
SET archived_at = i.predecessor_archived_at
FROM inserted i
WHERE p.id = i.predecessor_post_id AND p.archived_at IS NULL;

-- pass values-returning
WITH inserted AS (
  INSERT INTO repro_successions (predecessor_post_id, predecessor_archived_at)
  VALUES ($1::uuid, $2::timestamptz)
  ON CONFLICT (predecessor_post_id, predecessor_archived_at) DO NOTHING
  RETURNING predecessor_post_id, predecessor_archived_at
)
UPDATE repro_posts p
SET archived_at = i.predecessor_archived_at
FROM inserted i
WHERE p.id = i.predecessor_post_id AND p.archived_at IS NULL;

-- report cte-twice
WITH requested AS (
  SELECT predecessor_post_id FROM jsonb_to_recordset($1::jsonb) AS x(predecessor_post_id uuid)
),
inserted AS (
  INSERT INTO repro_successions (predecessor_post_id, predecessor_archived_at)
  SELECT r.predecessor_post_id, CURRENT_TIMESTAMP
  FROM requested r JOIN repro_posts p ON p.id = r.predecessor_post_id
  WHERE p.archived_at IS NULL
  ON CONFLICT (predecessor_post_id, predecessor_archived_at) DO NOTHING
  RETURNING predecessor_post_id, predecessor_archived_at
)
UPDATE repro_posts p
SET archived_at = i.predecessor_archived_at
FROM inserted i
JOIN inserted again ON again.predecessor_post_id = i.predecessor_post_id
WHERE p.id = i.predecessor_post_id AND p.archived_at IS NULL;

-- report or-join
WITH requested AS (
  SELECT predecessor_post_id FROM jsonb_to_recordset($1::jsonb) AS x(predecessor_post_id uuid)
),
inserted AS (
  INSERT INTO repro_successions (predecessor_post_id, predecessor_archived_at)
  SELECT r.predecessor_post_id, CURRENT_TIMESTAMP
  FROM requested r JOIN repro_posts p ON p.id = r.predecessor_post_id
  WHERE p.archived_at IS NULL
  ON CONFLICT (predecessor_post_id, predecessor_archived_at) DO NOTHING
  RETURNING predecessor_post_id, predecessor_archived_at
)
UPDATE repro_posts p
SET archived_at = i.predecessor_archived_at
FROM inserted i
WHERE p.id = i.predecessor_post_id OR p.archived_at IS NULL;

-- report nonunique-join
WITH requested AS (
  SELECT predecessor_post_id FROM jsonb_to_recordset($1::jsonb) AS x(predecessor_post_id uuid)
),
inserted AS (
  INSERT INTO repro_successions (predecessor_post_id, predecessor_archived_at)
  SELECT r.predecessor_post_id, CURRENT_TIMESTAMP
  FROM requested r JOIN repro_posts p ON p.id = r.predecessor_post_id
  WHERE p.archived_at IS NULL
  ON CONFLICT (predecessor_post_id, predecessor_archived_at) DO NOTHING
  RETURNING predecessor_post_id, predecessor_archived_at
)
UPDATE repro_posts p
SET archived_at = i.predecessor_archived_at
FROM inserted i
WHERE p.archived_at = i.predecessor_archived_at;

WITH epoch AS (
  SELECT ids FROM epochs
),
restored AS (
  UPDATE items SET archived_at = NULL
  FROM epoch
  CROSS JOIN LATERAL unnest(epoch.ids) AS u(id)
  WHERE items.id = u.id
  RETURNING items.id
)
SELECT id FROM restored

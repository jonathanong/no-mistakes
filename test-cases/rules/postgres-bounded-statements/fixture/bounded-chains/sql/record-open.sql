WITH req AS (
  SELECT input.item_id, input.link_id
  FROM events e
  CROSS JOIN LATERAL jsonb_to_recordset(e.payload) AS input(item_id uuid, link_id uuid)
),
term AS (
  UPDATE links l SET restored_at = now() FROM req
  WHERE l.id = req.link_id RETURNING l.item_id
),
restored AS (
  UPDATE items SET archived_at = NULL FROM term WHERE items.id = term.item_id RETURNING items.id
)
SELECT id FROM restored ORDER BY id

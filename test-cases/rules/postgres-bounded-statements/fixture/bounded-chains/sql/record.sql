WITH req AS (
  SELECT item_id, link_id FROM jsonb_to_recordset($1::jsonb) AS input(item_id uuid, link_id uuid)
),
term AS (
  UPDATE links l SET restored_at = now() FROM req
  WHERE l.id = req.link_id AND l.item_id = req.item_id RETURNING l.item_id
),
restored AS (
  UPDATE items SET archived_at = NULL FROM term WHERE items.id = term.item_id RETURNING items.id
)
SELECT id FROM restored ORDER BY id

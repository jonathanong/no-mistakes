SELECT id, order_index FROM questions WHERE community_id = $1 AND deleted_at IS NULL ORDER BY order_index;
UPDATE questions SET deleted_at = CURRENT_TIMESTAMP WHERE community_id = $1 AND deleted_at IS NULL;
WITH retired AS (
  UPDATE questions SET deleted_at = CURRENT_TIMESTAMP WHERE community_id = $1 AND deleted_at IS NULL RETURNING id
)
SELECT id FROM retired;
SELECT id, order_index FROM questions WHERE community_id = $1 ORDER BY order_index;
SELECT actor_id FROM participants WHERE conversation_id = $1;
SELECT e.id, ARRAY(SELECT p.actor_id FROM participants p WHERE p.conversation_id = e.id)
FROM entries e ORDER BY e.id DESC LIMIT 26;
SELECT day, metric, category FROM daily WHERE owner_id = $1
  AND day >= DATE '2026-01-01' AND day <= DATE '2026-12-31' ORDER BY day;
SELECT id FROM wide_slots WHERE owner_id = $1;
SELECT id FROM flags WHERE owner_id = $1;
SELECT id FROM ranks WHERE owner_id = $1;
SELECT id FROM maybe_flags WHERE owner_id = $1;
SELECT id FROM drafts WHERE owner_id = $1;
SELECT day FROM open_days WHERE owner_id = $1 AND day >= DATE '2026-01-01';
SELECT day FROM spans WHERE owner_id = $1 AND (
  day >= DATE '2026-01-01' AND day <= DATE '2026-02-09'
  OR day >= DATE '2027-01-01' AND day <= DATE '2027-02-09'
);
SELECT day FROM spans WHERE owner_id = $1 AND (
  day >= DATE '2026-01-01' AND day <= DATE '2034-03-19'
  OR day >= DATE '2035-01-01' AND day <= DATE '2043-03-19'
);
SELECT id FROM participants WHERE conversation_id = $1 AND slot = 1;
SELECT id FROM (SELECT actor_id AS id FROM participants WHERE conversation_id = $1 AND slot = 1) s;
SELECT id FROM not_a_table;
SELECT id FROM ranks WHERE owner_id = $1 AND rank >= 2 AND rank <= 4;
SELECT id FROM ranks WHERE owner_id = $1 AND rank >= 20 AND rank <= 30;
UPDATE wide_slots SET id = id WHERE owner_id = $1;
SELECT slug FROM slugs WHERE slug = $1;
SELECT actor_id FROM participants
WHERE conversation_id = $1 AND (slot = 1 OR slot = (SELECT NULL::integer));

SELECT t.id FROM a t JOIN plain_view v ON v.id = COALESCE(t.parent_id, t.id) WHERE t.id = $1;
SELECT t.id FROM a t JOIN lateral_view v ON v.id = COALESCE(t.parent_id, t.id) WHERE t.id = $1;
SELECT t.id FROM a t JOIN view_join v ON v.id = COALESCE(t.parent_id, t.id) WHERE t.id = $1;

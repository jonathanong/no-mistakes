SELECT t.id FROM a t JOIN plain_view v ON COALESCE(v.parent_id, v.id) = t.id WHERE t.id = $1;
SELECT t.id FROM a t JOIN lateral_view v ON COALESCE(v.parent_id, v.id) = t.id WHERE t.id = $1;
SELECT t.id FROM a t JOIN view_join v ON COALESCE(v.parent_id, v.id) = t.id WHERE t.id = $1;

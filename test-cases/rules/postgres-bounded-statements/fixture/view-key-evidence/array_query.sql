SELECT * FROM aliased WHERE visible_id = ANY(ARRAY(SELECT id FROM base WHERE id = $1));

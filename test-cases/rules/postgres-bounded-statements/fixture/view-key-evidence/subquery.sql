SELECT * FROM aliased WHERE visible_id = (SELECT id FROM base WHERE id = $1);

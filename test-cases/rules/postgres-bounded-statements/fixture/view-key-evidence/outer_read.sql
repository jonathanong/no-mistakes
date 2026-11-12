SELECT * FROM aliased WHERE visible_id = (SELECT id FROM base WHERE base.id = aliased.visible_id LIMIT 1);

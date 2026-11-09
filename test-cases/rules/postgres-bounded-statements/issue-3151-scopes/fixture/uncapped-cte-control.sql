WITH page AS (
  SELECT v.id FROM versions v
  WHERE v.id > $2
    AND (SELECT i.deleted_at IS NULL AND i.sha = v.sha FROM items i WHERE i.id = v.item_id) IS TRUE
  ORDER BY v.id
) SELECT * FROM page;

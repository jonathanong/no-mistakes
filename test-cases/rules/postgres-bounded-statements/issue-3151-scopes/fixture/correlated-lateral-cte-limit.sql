WITH page AS (
  SELECT v.id FROM versions v
  CROSS JOIN LATERAL (
    SELECT (SELECT i.deleted_at IS NULL AND i.sha = v.sha FROM items i WHERE i.id = v.item_id) AS live
  ) x
  WHERE v.id > $2 AND x.live IS TRUE
  ORDER BY v.id LIMIT $1 + 1
) SELECT * FROM page;

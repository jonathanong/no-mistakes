SELECT a.id, (SELECT label FROM kinds
  WHERE id = CASE WHEN random() > 0.5 THEN kind_id ELSE id END) AS label
FROM attachments a WHERE a.parent_id = $1;

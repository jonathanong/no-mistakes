SELECT a.id, (
  SELECT id FROM number_kinds WHERE id = kind_id
  UNION ALL
  SELECT id FROM number_kinds AS n(id, kind_id, label) WHERE id = kind_id
) AS id FROM attachments a WHERE a.parent_id = $1;

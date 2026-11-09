SELECT a.id, (SELECT k.label FROM kinds k CROSS JOIN unknown_kinds u WHERE k.id = kind_id) AS label
FROM attachments a WHERE a.parent_id = $1;

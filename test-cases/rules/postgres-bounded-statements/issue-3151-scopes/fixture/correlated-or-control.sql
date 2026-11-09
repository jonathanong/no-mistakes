SELECT a.id, (SELECT label FROM kinds WHERE id = kind_id OR id > 0) AS label
FROM attachments a WHERE a.parent_id = $1;

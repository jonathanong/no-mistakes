SELECT a.id, (SELECT label FROM kinds WHERE id = kind_id) AS label
FROM attachments a WHERE a.parent_id = $1;

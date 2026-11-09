SELECT a.id, (SELECT label FROM kinds WHERE label = note) AS label
FROM attachments a WHERE a.parent_id = $1;

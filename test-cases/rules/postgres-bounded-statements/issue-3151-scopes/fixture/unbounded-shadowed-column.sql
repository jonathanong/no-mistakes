SELECT a.id, (SELECT label FROM kind_shadows WHERE id = kind_id) AS label
FROM attachments a WHERE a.parent_id = $1;

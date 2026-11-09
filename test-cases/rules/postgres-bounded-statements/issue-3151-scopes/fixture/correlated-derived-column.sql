SELECT a.kind_id, (SELECT label FROM kinds WHERE id = kind_id) AS label
FROM (SELECT kind_id FROM attachments WHERE id = $1) a;

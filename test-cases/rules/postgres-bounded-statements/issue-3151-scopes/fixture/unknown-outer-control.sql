SELECT o.id, (SELECT label FROM kinds WHERE id = kind_id) AS label
FROM owners o CROSS JOIN unknown_attachments a WHERE o.id = $1;

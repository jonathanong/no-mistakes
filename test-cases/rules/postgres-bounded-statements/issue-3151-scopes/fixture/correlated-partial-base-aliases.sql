SELECT a.attachment_id, (SELECT label FROM kinds WHERE id = kind_id) AS label
FROM attachments AS a(attachment_id) WHERE a.parent_id = $1;

SELECT a.attachment_id, (SELECT label FROM kinds WHERE id = kind_key) AS label
FROM attachments AS a(attachment_id, parent_id, kind_key) WHERE a.parent_id = $1;

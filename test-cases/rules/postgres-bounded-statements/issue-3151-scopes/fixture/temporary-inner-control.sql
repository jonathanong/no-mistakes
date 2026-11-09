CREATE TEMP TABLE versions (kind_id integer);
SELECT a.id, (SELECT k.label FROM kinds k CROSS JOIN versions t WHERE k.id = kind_id) AS label
FROM attachments a WHERE a.parent_id = $1;

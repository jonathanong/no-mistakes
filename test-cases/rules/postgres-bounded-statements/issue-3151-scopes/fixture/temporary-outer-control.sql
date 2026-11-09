CREATE TEMP TABLE attachments (id integer, parent_id integer, kind_id integer);
SELECT COALESCE((SELECT label FROM kinds), 'missing') FROM attachments LIMIT 1;

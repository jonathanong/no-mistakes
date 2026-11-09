SELECT (SELECT label FROM kinds WHERE id = kind_id) AS label
FROM unnest($1::integer[]) AS ids(kind_id);

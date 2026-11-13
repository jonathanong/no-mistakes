SELECT * FROM (SELECT NULL::integer AS id FROM base UNION ALL SELECT id FROM related) d WHERE d.id = $1 OR d.id = $2;

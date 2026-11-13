SELECT * FROM (SELECT NULL::integer AS id FROM base UNION ALL SELECT tenant FROM base) d WHERE d.id = $1;

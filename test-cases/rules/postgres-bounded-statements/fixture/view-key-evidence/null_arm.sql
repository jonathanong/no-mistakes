SELECT * FROM (SELECT NULL::integer AS id FROM base) d WHERE d.id = $1;

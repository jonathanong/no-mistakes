SELECT * FROM (SELECT n.id FROM (SELECT NULL::integer AS id FROM base) n) d WHERE d.id = $1;

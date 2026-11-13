SELECT * FROM (SELECT NULL::integer AS id, tenant FROM base) d WHERE d.id = $1 OR d.tenant = $2;

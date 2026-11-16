SELECT owner_id FROM "audit.log" WHERE owner_id = $1;
SELECT owner_id FROM split WHERE owner_id = $1;

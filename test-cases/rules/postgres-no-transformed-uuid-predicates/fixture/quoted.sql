SELECT 1 FROM "Case" WHERE "ID"::text = $1;
SELECT 1 FROM case WHERE id::text = $1;
SELECT 1 FROM "Case" "A" WHERE "A"."ID"::text = $1;
SELECT 1 FROM public."Case" WHERE public."Case"."ID"::text = $1;
SELECT 1 FROM "dot.table" WHERE "dot.table".id::text = $1;
SELECT 1 FROM "odd.schema"."Mixed" "Alias" WHERE "Alias"."ID"::text = $1;
-- An unquoted identifier cannot alias a differently cased quoted name.
SELECT 1 FROM "Case" "A" WHERE a."ID"::text = $1;

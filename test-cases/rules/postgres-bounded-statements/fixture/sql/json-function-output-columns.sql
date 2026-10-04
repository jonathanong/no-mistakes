-- These built-ins retain their declared OUT name even with a relation alias.
SELECT value FROM json_array_elements($1::json) AS item;
SELECT value FROM json_array_elements_text($1::json) AS item;
SELECT value FROM jsonb_array_elements($1::jsonb) AS item;
SELECT value FROM jsonb_array_elements_text($1::jsonb) AS item;
SELECT value FROM jsonb_array_elements($1::jsonb);
SELECT value FROM pg_catalog.jsonb_array_elements($1::jsonb) AS item;
SELECT value FROM pg_catalog."jsonb_array_elements"($1::jsonb) AS item;
SELECT value, ordinality FROM jsonb_array_elements($1::jsonb) WITH ORDINALITY AS item;
SELECT renamed FROM jsonb_array_elements($1::jsonb) AS item(renamed);
-- Explicit output renaming hides the original OUT name.
SELECT value FROM jsonb_array_elements($1::jsonb) AS item(renamed);
-- Ordinary scalar table functions still take their output name from the alias.
SELECT item FROM generate_series(1, 3) AS item;

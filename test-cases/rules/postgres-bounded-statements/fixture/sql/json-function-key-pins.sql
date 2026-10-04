-- The local OUT column must not be mistaken for a read of the DELETE target.
DELETE FROM accounts WHERE id IN (SELECT value::text::uuid FROM json_array_elements($1::json) AS item);
DELETE FROM accounts WHERE id IN (SELECT value::uuid FROM json_array_elements_text($1::json) AS item);
DELETE FROM accounts WHERE id IN (SELECT value::text::uuid FROM jsonb_array_elements($1::jsonb) AS item);
DELETE FROM accounts WHERE id IN (SELECT value::uuid FROM jsonb_array_elements_text($1::jsonb) AS item);
DELETE FROM accounts WHERE id IN (SELECT renamed::uuid FROM pg_catalog.jsonb_array_elements_text($1::jsonb) AS item(renamed));
-- Functions in another schema retain opaque cardinality even with the same name.
DELETE FROM accounts WHERE id IN (SELECT value::uuid FROM app.jsonb_array_elements_text($1::jsonb) AS item);
-- A separate uncapped physical read remains an offender.
SELECT 1 FROM accounts, orders WHERE accounts.id IN (SELECT value::uuid FROM jsonb_array_elements_text($1::jsonb) AS item) AND orders.status='open';

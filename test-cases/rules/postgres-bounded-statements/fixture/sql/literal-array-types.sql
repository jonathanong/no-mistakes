-- Enum cast targets need catalog evidence; interval literals already have scalar cardinality.
DELETE FROM accounts WHERE email=ANY(ARRAY['active'::account_status]::text[]);
DELETE FROM accounts WHERE email=ANY(ARRAY['active'::app.text_domain]::text[]);
DELETE FROM accounts WHERE email=ANY(ARRAY['a'::ambiguous_status]::text[]);
DELETE FROM accounts WHERE email=ANY(ARRAY[INTERVAL '1 day']::text[]);
DELETE FROM accounts WHERE email=ANY(ARRAY['127.0.0.1'::pg_catalog.inet]::text[]);
DELETE FROM accounts WHERE email=ANY(ARRAY[coalesce('active'::account_status, 'inactive'::account_status)]::text[]);
DELETE FROM accounts WHERE email=ANY(ARRAY[('active'::account_status)::text[]]);
UPDATE accounts a SET name='x' FROM orders o WHERE o.id=$1 AND a.email=ANY(ARRAY[(o.encoded_ids::account_status)::text[]]);

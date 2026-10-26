-- DML in CTEs must remain visible, with schema-qualified target identities.
WITH written AS (
 INSERT INTO public.orders (details) VALUES (jsonb_build_object('accountId', $1)) RETURNING id
) SELECT id FROM written;
WITH written AS (
 UPDATE public.orders SET details = '{"accountId": "a1"}'::jsonb RETURNING id
) SELECT id FROM written;
INSERT INTO orders (details) VALUES
 ('{"accountId": "a1"}'::jsonb),
 (jsonb_build_object('accountId', $2));

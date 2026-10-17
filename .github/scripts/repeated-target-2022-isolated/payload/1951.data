EXPLAIN INSERT INTO public."café" (id, value)
SELECT 1, COALESCE(lower('λ'), CURRENT_TIMESTAMP)
ON CONFLICT (id) DO UPDATE SET value = EXCLUDED.value;
DO $body$
BEGIN
  EXECUTE 'INSERT INTO items (id, value) SELECT 2, COALESCE(lower(''λ''), CURRENT_TIMESTAMP) ON CONFLICT (id) DO UPDATE SET value = EXCLUDED.value;';
END
$body$;

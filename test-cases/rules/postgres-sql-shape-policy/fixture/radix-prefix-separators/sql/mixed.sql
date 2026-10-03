DO $$ BEGIN RAISE NOTICE 'migration'; END $$;
SELECT * FROM orders LIMIT 0o_17;
SELECT * FROM orders LIMIT 0b_10;

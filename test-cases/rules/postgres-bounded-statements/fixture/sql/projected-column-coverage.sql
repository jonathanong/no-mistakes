-- Known built-in names include qualified spellings and calls without table aliases.
DELETE FROM accounts WHERE id IN (SELECT id FROM pg_catalog.generate_series(1, 3) LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM pg_catalog.generate_series(1, 3) id LIMIT 1);
-- A user function has a record layout this syntactic projection deliberately treats as unknown.
DELETE FROM accounts WHERE id IN (SELECT id FROM LATERAL unknown_function() AS f LIMIT 1);

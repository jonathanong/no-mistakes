-- The EXECUTE wrapper is statically projected, so it is not a failed SQL statement.
DO $$ BEGIN EXECUTE 'CREATE TABLE accounts (id bigint PRIMARY KEY)'; END $$;

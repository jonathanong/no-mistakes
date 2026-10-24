COPY accounts (id) FROM STDIN;
1
\.
-- This static EXECUTE is already analyzed and must not retain its wrapper marker.
DO $$ BEGIN EXECUTE 'CREATE TABLE audit_log (id bigint PRIMARY KEY)'; END $$;

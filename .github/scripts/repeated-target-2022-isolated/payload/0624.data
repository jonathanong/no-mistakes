-- Ordering is a public string contract, including nested quoted SQL expressions.
CREATE INDEX plain_identity ON example (id);
CREATE INDEX only_identity ON ONLY example (id);
CREATE UNIQUE INDEX quoted_identity ON "App"."Accounts" USING btree (lower("DisplayName") DESC NULLS LAST) INCLUDE ("TenantID") WHERE "Enabled" = true;
CREATE INDEX nested_identity ON events ((CASE WHEN active THEN coalesce(score, 1.25) ELSE 0 END), (ARRAY[1, 2, 3])) WHERE payload IS NOT NULL;
CREATE INDEX escaped_identity ON "a""b" ((lower('a''b')), (id + 1)) WITH (fillfactor = 70);

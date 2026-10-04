-- Valid DO body takes the lenient path; COPY data still cannot hide later TABLE tokens.
DO $$ BEGIN PERFORM 1; END $$;
COPY input_table FROM STDIN;
'
\.
CREATE TEMP TABLE "Accounts"(id uuid);
SELECT NULL::uuid UNION ALL TABLE "Accounts" LIMIT 1;

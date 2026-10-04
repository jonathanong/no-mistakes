-- COPY data is not SQL: its unmatched quote must not hide later TABLE tokens.
COPY input_table FROM STDIN;
'
\.
CREATE TEMP TABLE "Accounts"(id uuid);
SELECT NULL::uuid UNION ALL TABLE "Accounts" LIMIT 1;

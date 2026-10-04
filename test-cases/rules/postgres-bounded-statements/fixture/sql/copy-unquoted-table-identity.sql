-- The same COPY data must not make an unquoted permanent namesake temporary.
COPY input_table FROM STDIN;
'
\.
CREATE TEMP TABLE "Accounts"(id uuid);
SELECT NULL::uuid UNION ALL TABLE Accounts LIMIT 1;

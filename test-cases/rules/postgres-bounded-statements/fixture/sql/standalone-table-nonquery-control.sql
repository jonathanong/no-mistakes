-- DDL and ordinary SELECT skip the standalone TABLE rewrite.
CREATE TABLE only (id uuid);
SELECT 1 FROM accounts;

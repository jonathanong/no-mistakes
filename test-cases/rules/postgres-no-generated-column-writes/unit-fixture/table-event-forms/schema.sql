CREATE TEMP TABLE IF NOT EXISTS public."quoted.table" (
  "Input" int,
  id int,
  calculated int GENERATED ALWAYS AS (combine(("Input"), public."quoted.table".id, 1)) STORED,
  stamp timestamptz GENERATED ALWAYS AS (CURRENT_TIMESTAMP) STORED,
  PRIMARY KEY ("Input")
);
CREATE TEMPORARY TABLE second (id int);
CREATE UNLOGGED TABLE third (id int);
ALTER TABLE IF EXISTS ONLY public."quoted.table" ADD COLUMN extra int;
DROP TABLE IF EXISTS public."quoted.table", second;

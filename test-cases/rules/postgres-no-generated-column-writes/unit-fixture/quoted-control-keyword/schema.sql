CREATE TABLE orders (id int, total int GENERATED ALWAYS AS (id) STORED);
DO $$
DECLARE "IF" integer;
BEGIN
  "IF" := 1;
  DROP TABLE orders;
  CREATE TABLE orders (id int, total int);
END
$$;

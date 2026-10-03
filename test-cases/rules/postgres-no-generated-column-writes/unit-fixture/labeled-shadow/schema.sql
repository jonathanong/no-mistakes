CREATE TABLE orders (id int, total int GENERATED ALWAYS AS (id) STORED);
DO $$
DECLARE ddl text := 'ALTER TABLE orders ADD COLUMN note text';
BEGIN
  <<inner>> DECLARE ddl text := 'DROP TABLE orders';
  BEGIN
    NULL;
  END;
  -- The outer binding is restored after the nested block, so this is the ALTER.
  EXECUTE ddl;
END
$$;

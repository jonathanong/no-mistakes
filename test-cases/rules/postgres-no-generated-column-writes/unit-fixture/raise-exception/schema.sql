CREATE TABLE orders (id int, total int GENERATED ALWAYS AS (id) STORED);
DO $$
BEGIN
  BEGIN
    RAISE EXCEPTION 'stop';
    -- Unreachable: the handler below catches the exception before this DROP.
    DROP TABLE orders;
  EXCEPTION WHEN OTHERS THEN NULL;
  END;
END
$$;

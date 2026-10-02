CREATE TABLE orders (id int);
DO $$ DECLARE "RETURN" text; BEGIN
  "RETURN" := 'still running';
  CREATE TABLE quoted_return_kept (id int);
END $$;
DO $$ BEGIN
  RETURN;
  DROP TABLE orders;
  EXECUTE 'DROP TABLE orders';
  CREATE TABLE unreachable_after_return (id int);
END $$;
DO $$ BEGIN
  BEGIN
    RETURN;
  END;
  DROP TABLE orders;
  EXECUTE 'DROP TABLE orders';
  CREATE TABLE unreachable_after_nested_return (id int);
END $$;
DO $$ BEGIN
  IF false THEN
    RETURN;
  END IF;
  DROP TABLE orders;
  CREATE TABLE uncertain_after_conditional_return (id int);
END $$;
ALTER TABLE orders ADD COLUMN after_return int;

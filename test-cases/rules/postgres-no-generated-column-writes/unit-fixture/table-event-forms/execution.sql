CREATE TABLE orders (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
-- Policy checks inspect dormant DDL; definite live schema history must ignore it.
CREATE FUNCTION dormant() RETURNS void LANGUAGE plpgsql AS $$
BEGIN
  DROP TABLE orders;
  CREATE TABLE dormant_generated (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
  EXECUTE 'DROP TABLE orders';
END;
$$;
CREATE PROCEDURE dormant_proc() LANGUAGE plpgsql AS $$
BEGIN
  ALTER TABLE orders ADD COLUMN dormant_column int;
  EXECUTE 'CREATE TABLE dormant_exec (id int)';
END;
$$;
DO LANGUAGE plpgsql $$
BEGIN
  IF false THEN
    DROP TABLE orders;
    ALTER TABLE orders ADD COLUMN conditional_column int;
    CREATE TABLE conditional_created (id int);
    EXECUTE 'DROP TABLE orders';
  ELSE
    DROP TABLE orders;
  END IF;
  CREATE TABLE immediate (id int);
  EXECUTE 'CREATE TABLE immediate_exec (id int)';
  FOR row IN SELECT 1 LOOP
    DROP TABLE orders;
  END LOOP;
  CASE WHEN false THEN
    DROP TABLE orders;
  END CASE;
  n := CASE WHEN true THEN 1 ELSE 2 END;
END;
$$;
DO 'BEGIN IF false THEN DROP TABLE orders; END IF; CREATE TABLE immediate_quoted (id int); END;';
ALTER TABLE orders ADD COLUMN actual_column int;

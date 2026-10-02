CREATE TABLE orders (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
DO $$ DECLARE ddl text; copied text; BEGIN
  ddl := 'ALTER TABLE orders ADD COLUMN note text';
  IF false THEN ddl := 'DROP TABLE orders'; END IF;
  EXECUTE ddl;
  copied := ddl;
  EXECUTE copied;
  ddl := 'ALTER TABLE orders ADD COLUMN note text';
  CASE WHEN false THEN ddl := 'DROP TABLE orders'; ELSE NULL; END CASE;
  EXECUTE ddl;
  ddl := 'ALTER TABLE orders ADD COLUMN note text';
  FOR counter IN 1..0 LOOP ddl := 'DROP TABLE orders'; END LOOP;
  EXECUTE ddl;
  ddl := 'ALTER TABLE orders ADD COLUMN note text';
  BEGIN PERFORM 1; EXCEPTION WHEN OTHERS THEN ddl := 'DROP TABLE orders'; END;
  EXECUTE ddl;
END $$;
-- Dormant routines still provide broad policy facts; they do not execute DDL.
CREATE FUNCTION dormant() RETURNS void LANGUAGE plpgsql AS $$ DECLARE ddl text; BEGIN
  ddl := 'CREATE TABLE dormant_fact (id int)';
  EXECUTE ddl;
END $$;

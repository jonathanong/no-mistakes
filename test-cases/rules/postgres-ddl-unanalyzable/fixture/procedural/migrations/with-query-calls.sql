CREATE TABLE with_query_rows (id integer PRIMARY KEY);
-- Query ownership must keep the CTE and RETURNING calls exactly once.
DO $$
DECLARE row_value record;
BEGIN
  FOR row_value IN WITH seed AS (SELECT abs(1) AS id)
    INSERT INTO with_query_rows SELECT id FROM seed RETURNING abs(id)
  LOOP NULL; END LOOP;
  FOR row_value IN WITH inserted AS (
    INSERT INTO with_query_rows VALUES (2) RETURNING abs(id) AS id)
    SELECT abs(id) FROM inserted
  LOOP NULL; END LOOP;
END
$$;
CREATE FUNCTION with_query_calls() RETURNS void LANGUAGE plpgsql AS $function$
DECLARE row_value record;
BEGIN
  FOR row_value IN WITH seed AS (SELECT abs(3) AS id)
    INSERT INTO with_query_rows SELECT id FROM seed RETURNING abs(id)
  LOOP NULL; END LOOP;
  FOR row_value IN WITH inserted AS (
    INSERT INTO with_query_rows VALUES (4) RETURNING abs(id) AS id)
    SELECT abs(id) FROM inserted
  LOOP NULL; END LOOP;
END
$function$;
-- Same-line calls outside the query interval must survive reconciliation.
DO $$ DECLARE row_value record; BEGIN PERFORM abs(5); FOR row_value IN WITH seed AS (SELECT abs(6) AS id) INSERT INTO with_query_rows SELECT id FROM seed RETURNING abs(id) LOOP NULL; END LOOP; PERFORM abs(7); END $$;
CREATE FUNCTION same_line_query_calls() RETURNS void LANGUAGE plpgsql AS $$ DECLARE row_value record; BEGIN PERFORM abs(8); FOR row_value IN WITH seed AS (SELECT abs(9) AS id) INSERT INTO with_query_rows SELECT id FROM seed RETURNING abs(id) LOOP NULL; END LOOP; PERFORM abs(10); END $$;

CREATE TABLE procedural_loop_rows (id integer PRIMARY KEY, label text NOT NULL);

-- Query FOR loops can consume DML RETURNING as well as SELECT results.
DO $$
DECLARE
  row_id integer;
BEGIN
  FOR row_id IN INSERT INTO procedural_loop_rows VALUES (1, 'one') RETURNING abs(id) LOOP
    NULL;
  END LOOP;
  FOR row_id IN UPDATE procedural_loop_rows SET label = 'updated' WHERE id = 1 RETURNING abs(id) LOOP
    NULL;
  END LOOP;
  FOR row_id IN DELETE FROM procedural_loop_rows WHERE id = 1 RETURNING abs(id) LOOP
    NULL;
  END LOOP;
END
$$;

CREATE FUNCTION procedural_dml_loop() RETURNS void LANGUAGE plpgsql AS $function$
DECLARE
  row_id integer;
BEGIN
  FOR row_id IN INSERT INTO procedural_loop_rows VALUES (2, 'two') RETURNING abs(id) LOOP
    NULL;
  END LOOP;
  FOR row_id IN UPDATE procedural_loop_rows SET label = 'updated' WHERE id = 2 RETURNING abs(id) LOOP
    NULL;
  END LOOP;
  FOR row_id IN DELETE FROM procedural_loop_rows WHERE id = 2 RETURNING abs(id) LOOP
    NULL;
  END LOOP;
END
$function$;

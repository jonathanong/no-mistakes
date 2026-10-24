CREATE TABLE grammar_error_rows (id integer PRIMARY KEY);

DO $$
DECLARE
  broken_declaration;
  missing_type := 1;
  bad_cursor_prefix NO SCROLL FOR SELECT id FROM grammar_error_rows;
  missing_cursor_for CURSOR SELECT id FROM grammar_error_rows;
  bad_collation text COLLATE;
  cursor_value refcursor;
  selected_id integer;
  row_value record;
BEGIN
  1 := 2;
  selected_id := ;
  GET DIAGNOSTICS ;
  OPEN ;
  OPEN cursor_value FOR ;
  MOVE TO FROM cursor_value;
  FETCH NEXT FROM cursor_value INTO ;
  FOR row_value SELECT id FROM grammar_error_rows LOOP
    NULL;
  END LOOP;
  FOR 1 IN 1..2 LOOP
    NULL;
  END LOOP;
  FOR row_value IN 1.. BY 2 LOOP
    NULL;
  END LOOP;
  FOR row_value IN 1..2 BY LOOP
    NULL;
  END LOOP;
  FOREACH 1 IN ARRAY ARRAY[1, 2] LOOP
    NULL;
  END LOOP;
  FOREACH row_value IN ARRAY LOOP
    NULL;
  END LOOP;
END
$$;

-- Recognized procedural wrappers do not hide malformed SQL payloads.
DO $$
DECLARE
  selected_id integer;
  cursor_value refcursor;
BEGIN
  OPEN cursor_value FOR FROBULATE accounts;
  EXECUTE 'FROBULATE accounts' INTO selected_id;
  SELECT 1 + ) INTO selected_id;
  GET DIAGNOSTICS selected_id = ROW_COUNT FROBULATE accounts;
  SELECT 1 INTO selected_id FROBULATE accounts;
END
$$;

CREATE TABLE malformed_cursor_rows (id integer PRIMARY KEY);

DO $$
DECLARE
  cursor_value refcursor;
BEGIN
  OPEN cursor_value NO SCROLL FOR SELECT id FROM malformed_cursor_rows FROBULATE accounts;
END
$$;

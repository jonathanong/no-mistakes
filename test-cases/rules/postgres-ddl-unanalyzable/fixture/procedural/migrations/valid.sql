-- Procedural delimiters and commands in these bodies are not failed SQL.
CREATE TABLE plpgsql_types (id integer PRIMARY KEY, label text NOT NULL);

DO $$
DECLARE
  branch integer := 0;
  assigned integer = 1;
  fixed CONSTANT integer := 2;
  required_value text NOT NULL := 'ready';
  default_value text DEFAULT 'default';
  typed_id plpgsql_types.id%TYPE := 1;
  row_value plpgsql_types%ROWTYPE;
  row_record record;
  static_cursor CURSOR FOR SELECT id FROM plpgsql_types;
  dynamic_cursor refcursor;
  dynamic_sql text := 'SELECT id FROM plpgsql_types';
  affected_count integer;
  diagnostic_text text;
  label_item text;
  index_value integer;
BEGIN
  IF true THEN
    CREATE TABLE audit_log (id bigint PRIMARY KEY);
    IF false THEN
      PERFORM 1;
    ELSE
      PERFORM 2;
    END IF;
  ELSIF false THEN
    RAISE NOTICE 'unreachable';
  ELSE
    PERFORM 3;
  END IF;
  IF (CASE WHEN true THEN true ELSE false END) THEN
    RAISE NOTICE 'case condition';
  END IF;
  LOOP
    EXIT WHEN true;
    CONTINUE;
  END LOOP;
  CASE branch
    WHEN 0 THEN PERFORM 4;
    ELSE RAISE NOTICE 'other';
  END CASE;
  SELECT id, label INTO row_value FROM plpgsql_types LIMIT 1;
  row_value.label := 'assigned';
  row_value.label = 'also assigned';
  SELECT id, label INTO row_record FROM plpgsql_types LIMIT 1;
  row_record.label := 'record field';
  row_record.label = 'record field assignment';
  OPEN static_cursor;
  FETCH NEXT FROM static_cursor INTO typed_id;
  CLOSE static_cursor;
  OPEN dynamic_cursor FOR SELECT id FROM plpgsql_types;
  FETCH NEXT FROM dynamic_cursor INTO typed_id;
  CLOSE dynamic_cursor;
  EXECUTE dynamic_sql INTO typed_id;
  GET DIAGNOSTICS affected_count = ROW_COUNT;
  WHILE branch < 1 LOOP
    branch := branch + 1;
  END LOOP;
  FOR index_value IN 1..0 LOOP
    NULL;
  END LOOP;
  FOR row_value IN SELECT id, label FROM plpgsql_types LOOP
    PERFORM row_value.id;
  END LOOP;
  FOREACH label_item IN ARRAY ARRAY['a', 'b'] LOOP
    PERFORM label_item;
  END LOOP;
  BEGIN
    RAISE EXCEPTION 'exercise stacked diagnostics';
  EXCEPTION WHEN others THEN
    GET STACKED DIAGNOSTICS diagnostic_text = MESSAGE_TEXT;
  END;
  <<outer_loop>>
  LOOP
    <<inner_block>>
    BEGIN
      EXIT outer_loop;
    END inner_block;
  END LOOP outer_loop;
  <<labeled_block>>
  BEGIN
    PERFORM 8;
  END labeled_block;
  RETURN;
END
$$;

CREATE FUNCTION audit_branch() RETURNS void LANGUAGE plpgsql AS $function$
DECLARE
  function_count integer = 0;
  function_fixed CONSTANT integer := 1;
  function_required text NOT NULL := 'function';
  function_default text DEFAULT 'default';
  function_id plpgsql_types.id%TYPE;
  function_row plpgsql_types%ROWTYPE;
  function_record record;
  function_cursor CURSOR FOR SELECT id FROM plpgsql_types;
  function_affected integer;
  function_message text;
BEGIN
  IF true THEN
    LOOP
      EXIT;
    END LOOP;
  ELSE
    RAISE NOTICE 'branch';
  END IF;
  CASE 1 WHEN 1 THEN PERFORM 5; ELSE PERFORM 6; END CASE;
  SELECT id, label INTO function_row FROM plpgsql_types LIMIT 1;
  function_row.label := 'changed';
  SELECT id, label INTO function_record FROM plpgsql_types LIMIT 1;
  function_record.label = 'changed';
  OPEN function_cursor;
  FETCH NEXT FROM function_cursor INTO function_id;
  CLOSE function_cursor;
  EXECUTE 'SELECT id FROM plpgsql_types' INTO function_id;
  GET DIAGNOSTICS function_affected = ROW_COUNT;
  BEGIN
    RAISE EXCEPTION 'exercise function diagnostics';
  EXCEPTION WHEN others THEN
    GET STACKED DIAGNOSTICS function_message = MESSAGE_TEXT;
  END;
  RETURN;
END
$function$;

CREATE PROCEDURE audit_procedure() LANGUAGE plpgsql AS $procedure$
BEGIN
  IF true THEN
    PERFORM 7;
  ELSE
    RAISE NOTICE 'procedure';
  END IF;
  RETURN;
END
$procedure$;

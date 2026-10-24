-- Unknown actions after each supported block retain their own locations.
DO $$
DECLARE
  counter_value integer := 0;
  item_value integer;
  error_text text;
BEGIN
  WHILE counter_value < 1 LOOP
    counter_value := counter_value + 1;
  END LOOP;
  FROBULATE after_while;
  FOR item_value IN 1..0 LOOP
    NULL;
  END LOOP;
  FROBULATE after_for;
  FOREACH item_value IN ARRAY ARRAY[1] LOOP
    NULL;
  END LOOP;
  FROBULATE after_foreach;
  BEGIN
    RAISE EXCEPTION 'probe';
  EXCEPTION WHEN others THEN
    GET STACKED DIAGNOSTICS error_text = MESSAGE_TEXT;
  END;
  FROBULATE after_exception;
END
$$;
FROBULATE after_block_do;

CREATE FUNCTION block_command_followups() RETURNS void LANGUAGE plpgsql AS $function$
DECLARE
  counter_value integer := 0;
  item_value integer;
  error_text text;
BEGIN
  WHILE counter_value < 1 LOOP
    counter_value := counter_value + 1;
  END LOOP;
  FROBULATE after_function_while;
  FOR item_value IN 1..0 LOOP
    NULL;
  END LOOP;
  FROBULATE after_function_for;
  FOREACH item_value IN ARRAY ARRAY[1] LOOP
    NULL;
  END LOOP;
  FROBULATE after_function_foreach;
  BEGIN
    RAISE EXCEPTION 'probe';
  EXCEPTION WHEN others THEN
    GET STACKED DIAGNOSTICS error_text = MESSAGE_TEXT;
  END;
  FROBULATE after_function_exception;
END
$function$;
FROBULATE after_block_function;

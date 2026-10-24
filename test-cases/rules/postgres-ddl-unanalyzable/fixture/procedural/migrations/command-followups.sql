-- Recovery after command-rich body wrappers keeps top-level siblings visible.
DO $$
DECLARE
  local_count integer := 0;
BEGIN
  WHILE local_count < 1 LOOP
    local_count := local_count + 1;
  END LOOP;
END
$$;
FROBULATE after_command_do;

CREATE FUNCTION command_followup() RETURNS void LANGUAGE plpgsql AS $function$
DECLARE
  local_count integer := 0;
BEGIN
  local_count := local_count + 1;
END
$function$;
FROBULATE after_command_function;

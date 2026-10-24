-- Procedural delimiters and commands in these bodies are not failed SQL.
DO $$
DECLARE
  branch integer := 0;
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
  IF CASE WHEN true THEN true ELSE false END THEN
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
BEGIN
  IF true THEN
    LOOP
      EXIT;
    END LOOP;
  ELSE
    RAISE NOTICE 'branch';
  END IF;
  CASE 1 WHEN 1 THEN PERFORM 5; ELSE PERFORM 6; END CASE;
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

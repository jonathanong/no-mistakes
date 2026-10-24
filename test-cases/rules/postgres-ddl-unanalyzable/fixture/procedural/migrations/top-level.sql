-- Recognized procedural commands inside this body are valid delimiters.
DO $$
BEGIN
  PERFORM 1;
END
$$;
FROBULATE after_do;
CREATE FUNCTION wrapper_function() RETURNS void LANGUAGE plpgsql AS $function$
BEGIN
  PERFORM 2;
END
$function$;
FROBULATE after_function;
CREATE PROCEDURE wrapper_procedure() LANGUAGE plpgsql AS $procedure$
BEGIN
  PERFORM 3;
END
$procedure$;
FROBULATE after_procedure;
-- Outside a PL/pgSQL body, this is an unknown control statement too.
END IF;

DO $$
DECLARE
  c CURSOR (n integer) FOR SELECT abs(n);
  selected_number integer;
BEGIN
  OPEN c(-1);
  FETCH NEXT FROM c INTO selected_number;
  CLOSE c;
END
$$;

CREATE FUNCTION cursor_first_parameter() RETURNS void LANGUAGE plpgsql AS $function$
DECLARE
  c CURSOR (n integer) FOR SELECT abs(n);
  selected_number integer;
BEGIN
  OPEN c(-1);
  FETCH NEXT FROM c INTO selected_number;
  CLOSE c;
END
$function$;

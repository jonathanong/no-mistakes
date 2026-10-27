CREATE FUNCTION short_fn() RETURNS void LANGUAGE plpgsql AS $function$
BEGIN
  CREATE INDEX do_indexxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx ON accounts (id);
END
$function$;

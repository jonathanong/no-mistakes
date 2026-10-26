-- no-mistakes-disable-file postgres-fk-index: intentionally unsupported routine body
CREATE FUNCTION ignored_body() RETURNS void LANGUAGE plpgsql AS $body$
BEGIN
  FROBULATE file_suppressed;
END
$body$;

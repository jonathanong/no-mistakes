DO $body$
BEGIN
  -- no-mistakes-disable-next-line postgres-fk-index: intentionally unsupported body command
  FROBULATE suppressed_next;
  FROBULATE suppressed_line; -- no-mistakes-disable-line postgres-fk-index: reviewed command
  FROBULATE remaining_body;
END
$body$;

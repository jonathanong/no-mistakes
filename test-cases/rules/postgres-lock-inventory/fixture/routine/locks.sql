CREATE FUNCTION reserve_job() RETURNS bigint LANGUAGE sql AS $body$
  SELECT id FROM jobs FOR UPDATE;
$body$;

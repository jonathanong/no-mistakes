-- lock-reason: prevents: parse error should still be unanalyzable; writer: worker
SELECT FROM ???;

-- lock-reason: prevents: routine body should still be unanalyzable; writer: worker
CREATE FUNCTION reserve_job() RETURNS bigint LANGUAGE sql AS $body$
  SELECT id FROM jobs FOR UPDATE;
$body$;

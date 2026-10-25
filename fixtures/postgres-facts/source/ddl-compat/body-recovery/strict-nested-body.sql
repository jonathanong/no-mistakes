DO $body$
DECLARE
    result_ids bigint[] := ARRAY[0, 0];
    second_id bigint;
BEGIN
    SELECT abs(1) INTO STRICT result_ids[array_position(ARRAY[1, 2], 1)] FROM accounts WHERE id = 1;
    SELECT abs(2) INTO STRICT result_ids[EXTRACT(DAY FROM now())::int], second_id FROM accounts WHERE id = 2;
END
$body$;

SELECT abs(1) INTO STRICT result_ids[array_position(ARRAY[1, 2], 1)] FROM accounts WHERE id = 1;
SELECT abs(2) INTO STRICT result_ids[EXTRACT(DAY FROM now())::int], second_id FROM accounts WHERE id = 2;

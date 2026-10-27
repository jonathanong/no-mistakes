DELETE FROM accounts WHERE id = ANY(ARRAY[gen_random_uuid()]);

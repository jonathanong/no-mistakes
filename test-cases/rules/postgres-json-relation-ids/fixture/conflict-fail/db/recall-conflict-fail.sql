-- The insert arm's FK write does not repair a separate conflict UPDATE arm.
INSERT INTO orders (id, account_id, details)
VALUES ($1, $2, jsonb_build_object('accountId', $2))
ON CONFLICT (id) DO UPDATE SET details = jsonb_build_object('accountId', $3);

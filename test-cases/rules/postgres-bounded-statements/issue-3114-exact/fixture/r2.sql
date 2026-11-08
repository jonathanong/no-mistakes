WITH selected AS MATERIALIZED (SELECT unnest($1::uuid[]) AS id)
UPDATE auth_sessions SET revoked_at = CURRENT_TIMESTAMP
WHERE id = ANY(ARRAY(SELECT id FROM selected)) AND user_id = $2 AND revoked_at IS NULL
RETURNING id;

WITH selected AS MATERIALIZED (SELECT unnest($1::uuid[]) AS id)
UPDATE auth_sessions SET revoked_at = CURRENT_TIMESTAMP
WHERE id = ANY($1::uuid[]) AND user_id = $2 AND revoked_at IS NULL
RETURNING id;

WITH selected AS MATERIALIZED (SELECT id FROM auth_sessions)
UPDATE auth_sessions SET revoked_at = CURRENT_TIMESTAMP
WHERE id = ANY(ARRAY(SELECT id FROM selected)) AND user_id = $1 AND revoked_at IS NULL
RETURNING id;

WITH selected AS MATERIALIZED (SELECT unnest(opaque_ids()::uuid[]) AS id)
UPDATE auth_sessions SET revoked_at = CURRENT_TIMESTAMP
WHERE id = ANY(ARRAY(SELECT id FROM selected)) AND user_id = $1
RETURNING id;

WITH selected AS MATERIALIZED (SELECT app.unnest($1::uuid[]) AS id)
UPDATE auth_sessions SET revoked_at = CURRENT_TIMESTAMP
WHERE id = ANY(ARRAY(SELECT id FROM selected)) AND user_id = $2 AND revoked_at IS NULL
RETURNING id;

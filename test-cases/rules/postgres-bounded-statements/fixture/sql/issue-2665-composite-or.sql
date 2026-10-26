-- Each branch completes a different real three-column unique key.
SELECT * FROM decisions WHERE tenant_id = $1 AND scope = $2 AND (decision_id = $3 OR idempotency_key = $4);
-- Non-key and incomplete alternatives cannot borrow key components from other branches.
SELECT * FROM decisions WHERE tenant_id = $1 AND scope = $2 AND (decision_id = $3 OR status = $4);
SELECT * FROM decisions WHERE (tenant_id = $1 AND decision_id = $2) OR (scope = $3 AND idempotency_key = $4);

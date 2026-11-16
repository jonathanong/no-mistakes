SELECT id, user_id, consent_type, revoked_at FROM tail_consents WHERE user_id = $1 AND revoked_at IS NULL;
SELECT batch.id, COUNT(import_row.id)::int
FROM tail_import_batches batch
LEFT JOIN tail_import_rows import_row ON import_row.batch_id = batch.id
WHERE batch.id = $1 AND batch.user_id = $2
GROUP BY batch.id;

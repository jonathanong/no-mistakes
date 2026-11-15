-- A limited candidate set joined on the primary key cannot fan out.
WITH targets AS (SELECT id FROM flows WHERE status = 'pending' ORDER BY id LIMIT 25)
UPDATE flows SET rejected_at = now() FROM targets WHERE flows.id = targets.id;

-- status is not unique, so the same limit can match many flows.
WITH targets AS (SELECT status FROM flows WHERE id IS NOT NULL ORDER BY status LIMIT 25)
UPDATE flows SET rejected_at = now() FROM targets WHERE flows.status = targets.status;

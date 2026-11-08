WITH updated_row AS (UPDATE input_rows SET error_message = NULL WHERE id = $1 RETURNING batch_id),
updated_batch AS (
  UPDATE work_batches SET completed_rows = completed_rows + 1
  FROM updated_row WHERE work_batches.id = updated_row.batch_id
  RETURNING work_batches.id
)
SELECT * FROM updated_batch;

WITH updated_row AS (UPDATE input_rows SET error_message = NULL WHERE id = $1 RETURNING batch_id)
UPDATE work_batches SET completed_rows = completed_rows + 1
FROM updated_row WHERE work_batches.id = updated_row.batch_id;

WITH updated_row AS (UPDATE input_rows SET error_message = NULL WHERE error_message = $1 RETURNING batch_id),
updated_batch AS (
  UPDATE work_batches SET completed_rows = completed_rows + 1
  FROM updated_row WHERE work_batches.id = updated_row.batch_id
  RETURNING work_batches.id
)
SELECT * FROM updated_batch;

WITH candidates AS (SELECT id FROM work_batches ORDER BY id LIMIT $1),
changed AS (
  UPDATE work_batches SET completed_rows = completed_rows + 1
  FROM candidates WHERE work_batches.id = candidates.id RETURNING work_batches.id
)
SELECT * FROM changed;

WITH candidate AS (SELECT completed_rows FROM work_batches WHERE id = $1),
changed AS (
  UPDATE work_batches SET completed_rows = work_batches.completed_rows + 1
  FROM candidate WHERE work_batches.completed_rows = candidate.completed_rows
  RETURNING work_batches.id
)
SELECT * FROM changed;

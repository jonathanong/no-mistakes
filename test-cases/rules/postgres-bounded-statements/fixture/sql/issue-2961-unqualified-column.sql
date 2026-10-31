-- The CTE projects email only, so bare id belongs to the target primary key.
WITH one AS (SELECT email FROM accounts ORDER BY email LIMIT 1)
UPDATE orders SET account_id = $2 FROM one WHERE id = $1;

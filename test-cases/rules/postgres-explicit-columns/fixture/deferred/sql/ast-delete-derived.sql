-- A derived target must never be classified as a catalog table.
DELETE FROM (SELECT id FROM orders) t RETURNING *;

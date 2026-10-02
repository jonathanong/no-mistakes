UPDATE tags SET id = 1 FROM orders o RETURNING o.*;
DELETE FROM tags USING orders o RETURNING o.*;
UPDATE tags SET id = 1 FROM orders o RETURNING *;
DELETE FROM orders o RETURNING o.*;

WITH commented AS (
 INSERT INTO target (id) VALUES (1)
 ON CONFLICT (id) /* target */ WHERE id > 0 DO NOTHING RETURNING id
)
SELECT id FROM commented;

WITH quoted AS (
 INSERT INTO target (id) VALUES (1)
 ON CONFLICT (id) WHERE note = 'DO WHERE ) ; --' DO NOTHING RETURNING id
)
SELECT id FROM quoted;

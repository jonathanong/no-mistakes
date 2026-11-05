SELECT i.id FROM items i JOIN LATERAL (SELECT i.id AS id) q ON i.id = q.id;

SELECT i.id FROM external e JOIN items i ON i.id = e.item_id;

WITH inputs AS (SELECT * FROM app.rows())
SELECT u.id FROM inputs q JOIN users u ON u.id = q.user_id;

SELECT i.id
FROM items i
CROSS JOIN feeds f
CROSS JOIN topics t
CROSS JOIN accounts a
CROSS JOIN users u;

SELECT i.id
FROM items i
-- no-mistakes-disable-next-line postgres-bounded-statements -- reviewed finite feed relation
CROSS JOIN feeds f
CROSS JOIN topics t;

-- no-mistakes-disable-next-line postgres-bounded-statements -- reviewed whole statement
SELECT i.id FROM items i CROSS JOIN feeds f;

SELECT i.id FROM items i CROSS JOIN feeds f WHERE i.feed_id = $1;
SELECT i.id FROM items i CROSS JOIN topics t WHERE i.feed_id = $1;
SELECT id FROM items; SELECT id FROM items;
SELECT i.id FROM items i CROSS JOIN activity a;

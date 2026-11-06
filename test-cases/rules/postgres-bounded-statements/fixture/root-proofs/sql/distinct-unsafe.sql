SELECT DISTINCT ON (i.url_id) i.url_id FROM items i WHERE i.url_id = $1 GROUP BY ROLLUP(i.url_id) ORDER BY i.url_id;

SELECT DISTINCT ON (url_id) i.id AS url_id FROM items i WHERE i.url_id = $1 ORDER BY url_id;

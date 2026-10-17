-- Internal projection remains conservative if the parser later admits extensions.
SELECT * FROM t LIMIT 1, 2;
SELECT * FROM t LIMIT 2 BY id;
SELECT TOP 1 * FROM t;
SELECT * FROM t PREWHERE id = 1;
SELECT * FROM t QUALIFY id = 1;

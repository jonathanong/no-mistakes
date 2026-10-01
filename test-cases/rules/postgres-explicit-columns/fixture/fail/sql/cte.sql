WITH recent AS (SELECT * FROM orders WHERE id > $1) SELECT id FROM recent;

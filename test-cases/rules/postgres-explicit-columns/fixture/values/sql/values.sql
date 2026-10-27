INSERT INTO logs (payload) VALUES ((SELECT json_agg(r) FROM (SELECT * FROM orders) r));

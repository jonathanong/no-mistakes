-- Nested keys, dynamic keys, non-jsonb values, and reads are out of scope.
INSERT INTO orders (details) VALUES ('{"nested": {"accountId": "a1"}, "total": 10}'::jsonb);
INSERT INTO orders (details) VALUES (jsonb_build_object('nested', jsonb_build_object('accountId', $1)));
INSERT INTO orders (details) VALUES (jsonb_build_object($1, $2));
INSERT INTO orders (note) VALUES ('{"accountId": "a1"}');
SELECT jsonb_build_object('accountId', $1);
DELETE FROM orders WHERE details->>'accountId' = $1;

-- no-mistakes-disable-file postgres-json-relation-ids
INSERT INTO orders (details) VALUES (jsonb_build_object('accountId', $1));

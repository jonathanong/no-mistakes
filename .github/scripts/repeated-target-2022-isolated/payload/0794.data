-- The outer WITH wrapper must not drop the inner INSERT's conflict key.
WITH seed AS (SELECT 1)
INSERT INTO users (email) VALUES ('a')
ON CONFLICT (lower(email)) DO UPDATE SET email = 'b';

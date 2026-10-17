-- The invalid neighbor forces lenient recovery without changing the first INSERT.
INSERT INTO users (email) VALUES ('a')
ON CONFLICT (lower(email)) DO UPDATE SET email = 'b';
SELECT * FROM ;

INSERT INTO users (email, name) VALUES ('a', 'n')
ON CONFLICT (lower(email)) DO UPDATE SET name = 'b';

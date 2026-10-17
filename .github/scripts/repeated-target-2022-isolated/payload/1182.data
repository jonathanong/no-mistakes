INSERT INTO users (email, name) VALUES ('a', 'n')
ON CONFLICT (email, lower(name)) DO UPDATE SET name = 'b';

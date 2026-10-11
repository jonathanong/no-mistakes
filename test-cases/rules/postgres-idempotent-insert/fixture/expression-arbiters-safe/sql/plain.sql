INSERT INTO users (email) VALUES ('a')
ON CONFLICT (email) DO UPDATE SET email = EXCLUDED.email;

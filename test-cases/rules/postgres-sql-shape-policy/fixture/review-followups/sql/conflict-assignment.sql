INSERT INTO accounts (id) VALUES (1)
ON CONFLICT (id) DO UPDATE
SET blocked = excluded.id NOT IN (SELECT id FROM bans),
    quiet = (SELECT COUNT(*) FROM orders) = 0;

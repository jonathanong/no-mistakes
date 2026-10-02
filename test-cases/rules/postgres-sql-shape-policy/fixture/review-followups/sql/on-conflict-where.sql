INSERT INTO accounts (id) VALUES (1)
ON CONFLICT (id) DO UPDATE SET id = EXCLUDED.id
WHERE id NOT IN (
  SELECT account_id FROM bans
  WHERE (SELECT COUNT(*) FROM archived) > 0
);

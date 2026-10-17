-- Derived write targets accepted by the parser have no named table identity.
UPDATE (SELECT id FROM users) u SET id = 1;
MERGE INTO (SELECT id FROM users) u USING accounts a ON u.id = a.id
WHEN MATCHED THEN UPDATE SET id = a.id;

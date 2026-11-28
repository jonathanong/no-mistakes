WITH target_orders AS (
  SELECT id FROM orders WHERE status = 'pending'
)
UPDATE orders o
SET
  cached_name = (SELECT a1.display_name FROM accounts a1 WHERE a1.id = o.account_id),
  cached_tier = (SELECT a2.tier FROM accounts a2 WHERE a2.id = o.account_id)
FROM target_orders t
WHERE o.id = t.id;

WITH removed AS (
  DELETE FROM orders o
  WHERE (SELECT a1.suspended FROM accounts a1 WHERE a1.id = o.account_id)
    AND (SELECT a2.archived FROM accounts a2 WHERE a2.id = o.account_id)
  RETURNING o.id
)
SELECT id FROM removed;

MERGE INTO orders o
USING incoming_orders s
ON o.id = s.id
WHEN MATCHED THEN
  UPDATE SET
    cached_name = (SELECT a1.display_name FROM accounts a1 WHERE a1.id = s.account_id),
    cached_tier = (SELECT a2.tier FROM accounts a2 WHERE a2.id = s.account_id);

UPDATE orders AS o SET account_id = 42
RETURNING
    (SELECT a.name FROM accounts a WHERE a.id = NEW.account_id),
    (SELECT b.email FROM accounts b WHERE b.id = o.account_id),
    (SELECT c.phone FROM accounts c WHERE c.id = OLD.account_id);

UPDATE orders AS new SET account_id = 42
RETURNING
    (SELECT a.name FROM accounts a WHERE a.id = new.account_id),
    (SELECT b.email FROM accounts b WHERE b.id = new.account_id);

INSERT INTO orders AS o (account_id) VALUES (42)
RETURNING
    (SELECT a.name FROM accounts a WHERE a.id = NEW.account_id),
    (SELECT b.email FROM accounts b WHERE b.id = o.account_id);

DELETE FROM orders AS o
WHERE o.id = 7
RETURNING
    (SELECT b.name FROM accounts b WHERE b.id = OLD.account_id),
    (SELECT c.email FROM accounts c WHERE c.id = o.account_id);

MERGE INTO orders AS o USING pending p ON o.id = p.order_id
WHEN MATCHED THEN UPDATE SET account_id = p.account_id
RETURNING
    (SELECT a.name FROM accounts a WHERE a.id = OLD.account_id),
    (SELECT b.email FROM accounts b WHERE b.id = OLD.account_id),
    (SELECT c.phone FROM accounts c WHERE c.id = NEW.account_id);

MERGE INTO orders o USING incoming s ON o.id = s.id
WHEN MATCHED AND FALSE THEN UPDATE SET
    account_name = (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    account_tier = (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
WHEN MATCHED AND NULL THEN UPDATE SET
    account_name = (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    account_tier = (SELECT b.tier FROM accounts b WHERE b.id = o.account_id);

UPDATE orders AS o
SET account_name = (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    account_tier = (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
WHERE FALSE
RETURNING (SELECT c.name FROM accounts c WHERE c.id = o.account_id),
          (SELECT d.tier FROM accounts d WHERE d.id = o.account_id);

DELETE FROM orders AS o WHERE NULL
RETURNING (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
          (SELECT b.tier FROM accounts b WHERE b.id = o.account_id);

INSERT INTO orders AS o (id) VALUES (42)
ON CONFLICT (id) DO UPDATE SET
    account_name = (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    account_tier = (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
WHERE FALSE
RETURNING (SELECT c.name FROM accounts c WHERE c.id = o.account_id),
          (SELECT d.tier FROM accounts d WHERE d.id = o.account_id);

MERGE INTO orders o USING incoming s ON o.id = s.id
WHEN MATCHED AND TRUE THEN DO NOTHING
WHEN MATCHED AND (SELECT a.enabled FROM accounts a WHERE a.id = o.account_id)
    THEN UPDATE SET account_name = (SELECT b.name FROM accounts b WHERE b.id = o.account_id)
WHEN NOT MATCHED THEN INSERT (account_name, account_tier) VALUES (
    (SELECT c.name FROM accounts c WHERE c.id = s.account_id),
    (SELECT d.tier FROM accounts d WHERE d.id = s.account_id)
);

WITH touched AS (
    UPDATE orders o SET
        account_name = (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
        account_tier = (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
    RETURNING o.id
)
SELECT
    (SELECT a.name FROM accounts a WHERE a.id = touched.id),
    (SELECT b.tier FROM accounts b WHERE b.id = touched.id)
FROM touched
LIMIT 0 OFFSET (
    SELECT (SELECT a.n FROM accounts a WHERE a.id = $1)
         + (SELECT b.m FROM accounts b WHERE b.id = $1)
);

MERGE INTO orders o USING incoming s ON FALSE
WHEN MATCHED THEN UPDATE SET
    account_name = (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    account_tier = (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
WHEN NOT MATCHED THEN INSERT (account_name, account_tier) VALUES (
    (SELECT c.name FROM accounts c WHERE c.id = s.account_id),
    (SELECT d.tier FROM accounts d WHERE d.id = s.account_id)
);

MERGE INTO orders o USING incoming s ON NULL
WHEN MATCHED THEN UPDATE SET
    account_name = (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
    account_tier = (SELECT b.tier FROM accounts b WHERE b.id = o.account_id)
WHEN NOT MATCHED BY SOURCE THEN DO NOTHING
RETURNING (SELECT c.name FROM accounts c WHERE c.id = o.account_id),
          (SELECT d.tier FROM accounts d WHERE d.id = o.account_id);

MERGE INTO orders o USING incoming s ON o.id = s.id
WHEN MATCHED THEN DO NOTHING
RETURNING (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
          (SELECT b.tier FROM accounts b WHERE b.id = o.account_id);

MERGE INTO orders o USING incoming s ON o.id = s.id
WHEN MATCHED THEN DELETE
WHEN NOT MATCHED THEN INSERT (id) VALUES (DEFAULT)
RETURNING (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
          (SELECT b.tier FROM accounts b WHERE b.id = o.account_id);

INSERT INTO orders AS o DEFAULT VALUES ON CONFLICT DO NOTHING
RETURNING (SELECT a.name FROM accounts a WHERE a.id = o.account_id),
          (SELECT b.tier FROM accounts b WHERE b.id = o.account_id);

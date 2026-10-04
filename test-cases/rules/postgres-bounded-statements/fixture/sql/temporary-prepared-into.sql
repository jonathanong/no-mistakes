-- PREPARE parses the SELECT INTO but does not create its temporary target.
PREPARE make_accounts AS SELECT id INTO TEMP accounts FROM orders LIMIT 1;
SELECT * FROM accounts;
EXECUTE make_accounts;
SELECT * FROM accounts;
DEALLOCATE make_accounts;
DROP TABLE accounts;
SELECT * FROM accounts;

-- A prepared destination that is never executed cannot shadow the catalog.
PREPARE later_accounts AS SELECT id INTO TEMP accounts FROM orders LIMIT 1;
DEALLOCATE later_accounts;
SELECT * FROM accounts;

-- DEALLOCATE ALL clears earlier prepared definitions before a fresh one runs.
PREPARE abandoned AS SELECT id INTO TEMP accounts FROM orders LIMIT 1;
DEALLOCATE ALL;
PREPARE final_accounts AS SELECT id INTO TEMP accounts FROM orders LIMIT 1;
EXECUTE final_accounts;
SELECT * FROM accounts;

-- PostgreSQL rejects a duplicate PREPARE and keeps the first definition.
PREPARE p AS SELECT id FROM orders LIMIT 1;
PREPARE p AS SELECT id INTO TEMP accounts FROM orders LIMIT 1;
EXECUTE p;
SELECT * FROM accounts;
DEALLOCATE p;

-- The rejected duplicate cannot erase a prepared temporary destination either.
PREPARE p AS SELECT id INTO TEMP accounts FROM orders LIMIT 1;
PREPARE p AS SELECT id FROM orders LIMIT 1;
EXECUTE p;
SELECT * FROM accounts;
DROP TABLE accounts;
DEALLOCATE p;

-- A rejected EXECUTE with the wrong arity must not create the destination.
PREPARE p(uuid) AS SELECT id INTO TEMP accounts FROM orders WHERE id = $1 LIMIT 1;
EXECUTE p;
SELECT * FROM accounts;
EXECUTE p('00000000-0000-0000-0000-000000000000');
SELECT * FROM accounts;

-- PostgreSQL can infer parameter types when PREPARE omits a type list.
DEALLOCATE p;
DROP TABLE accounts;
PREPARE inferred AS SELECT id INTO TEMP accounts FROM orders WHERE id = $1 LIMIT 1;
EXECUTE inferred;
SELECT * FROM accounts;
EXECUTE inferred('00000000-0000-0000-0000-000000000000');
SELECT * FROM accounts;

-- PostgreSQL rejects parameter zero; it must not create a temporary identity.
DROP TABLE accounts;
PREPARE invalid AS SELECT id INTO TEMP accounts FROM orders WHERE id = $0 LIMIT 1;
EXECUTE invalid;
SELECT * FROM accounts;

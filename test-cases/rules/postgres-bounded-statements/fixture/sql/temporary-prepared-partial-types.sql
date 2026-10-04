-- PostgreSQL infers later parameter types beyond an explicitly declared prefix.
PREPARE partial(int) AS SELECT id INTO TEMP accounts FROM orders WHERE $1 > 0 AND $2::int > 0 LIMIT 1;
EXECUTE partial(1);
SELECT * FROM accounts;
EXECUTE partial(1, 2, 3);
SELECT * FROM accounts;
EXECUTE partial(1, 2);
SELECT * FROM accounts;
DROP TABLE accounts;
DEALLOCATE partial;

-- Unused declared parameters still count toward EXECUTE arity.
PREPARE declared(int, text, int) AS SELECT id INTO TEMP accounts FROM orders WHERE $1 > 0 LIMIT 1;
EXECUTE declared(1);
SELECT * FROM accounts;
EXECUTE declared(1, 'unused', 3);
SELECT * FROM accounts;
DROP TABLE accounts;
DEALLOCATE declared;

-- An invalid placeholder remains invalid even with a declared type prefix.
PREPARE invalid(int) AS SELECT id INTO TEMP accounts FROM orders WHERE $2::int > 0 AND $0 > 0 LIMIT 1;
EXECUTE invalid(1, 2);
SELECT * FROM accounts;

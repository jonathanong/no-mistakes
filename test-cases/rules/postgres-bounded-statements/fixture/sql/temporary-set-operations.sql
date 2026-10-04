-- Set-operation INTO targets shadow the permanent name only after reading their sources.
SELECT id INTO TEMP accounts FROM orders LIMIT 1;
DROP TABLE accounts;
SELECT id INTO TEMP accounts FROM orders UNION SELECT id FROM orders;
SELECT * FROM accounts;
DROP TABLE accounts;
SELECT * FROM accounts;
SELECT id INTO TEMP accounts FROM orders INTERSECT SELECT id FROM orders;
SELECT * FROM accounts;
DROP TABLE accounts;
SELECT id INTO TEMP accounts FROM orders EXCEPT SELECT id FROM orders;
SELECT * FROM accounts;
DROP TABLE accounts;
(SELECT id INTO TEMP accounts FROM orders) UNION SELECT id FROM orders;
SELECT * FROM accounts;
DROP TABLE accounts;
SELECT * FROM accounts;
VALUES (1);

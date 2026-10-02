-- ClickHouse table-function insertion exercises the shared non-base-target guard.
INSERT INTO FUNCTION orders() VALUES (1);

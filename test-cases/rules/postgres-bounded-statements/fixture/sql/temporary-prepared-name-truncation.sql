-- Unquoted names fold case, then share PostgreSQL's 63-byte identifier prefix.
PREPARE pppppppppppppppppppppppppppppppppppppppppppppppppppppppppppppppdeclared AS SELECT id INTO TEMP accounts FROM orders LIMIT 1;
EXECUTE PPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPexecuted;
SELECT * FROM accounts;
DROP TABLE accounts;
DEALLOCATE pppppppppppppppppppppppppppppppppppppppppppppppppppppppppppppppcleanup;
EXECUTE pppppppppppppppppppppppppppppppppppppppppppppppppppppppppppppppdeclared;
SELECT * FROM accounts;
-- Colliding PREPARE names keep the original definition.
PREPARE pppppppppppppppppppppppppppppppppppppppppppppppppppppppppppppppfirst AS SELECT id FROM orders LIMIT 1;
PREPARE pppppppppppppppppppppppppppppppppppppppppppppppppppppppppppppppsecond AS SELECT id INTO TEMP accounts FROM orders LIMIT 1;
EXECUTE pppppppppppppppppppppppppppppppppppppppppppppppppppppppppppppppthird;
SELECT * FROM accounts;
DEALLOCATE pppppppppppppppppppppppppppppppppppppppppppppppppppppppppppppppfourth;
-- Quoted case remains significant even when both identifiers are truncated.
PREPARE "QQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQfirst" AS SELECT id INTO TEMP accounts FROM orders LIMIT 1;
EXECUTE "qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqwrong";
SELECT * FROM accounts;
EXECUTE "QQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQsecond";
SELECT * FROM accounts;
DROP TABLE accounts;
DEALLOCATE "QQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQQcleanup";
-- A multi-byte character crossing byte 63 is omitted in its entirety.
PREPARE "ssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssss界first" AS SELECT id INTO TEMP accounts FROM orders LIMIT 1;
EXECUTE "ssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssss語second";
SELECT * FROM accounts;
DROP TABLE accounts;
DEALLOCATE "ssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssss文cleanup";
EXECUTE "ssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssss界first";
SELECT * FROM accounts;
-- A multi-byte character ending exactly at byte 63 remains part of the key.
PREPARE "nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnéfirst" AS SELECT id INTO TEMP accounts FROM orders LIMIT 1;
EXECUTE "nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnêwrong";
SELECT * FROM accounts;
EXECUTE "nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnésecond";
SELECT * FROM accounts;
DROP TABLE accounts;
DEALLOCATE "nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnécleanup";

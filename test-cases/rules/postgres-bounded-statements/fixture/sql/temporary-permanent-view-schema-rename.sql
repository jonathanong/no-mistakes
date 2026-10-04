-- A schema rename moves both the permanent view node and its source edge.
CREATE VIEW audit.middle AS SELECT * FROM audit.accounts;
CREATE TEMP VIEW orders AS SELECT * FROM audit.middle;
ALTER SCHEMA audit RENAME TO archived;
DROP TABLE audit.accounts CASCADE;
SELECT * FROM orders;
DROP TABLE archived.accounts CASCADE;
SELECT * FROM orders;

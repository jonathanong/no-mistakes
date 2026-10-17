CREATE RECURSIVE VIEW recursive_view(x) AS SELECT 3 AS x;
CREATE OR REPLACE RECURSIVE VIEW "App"."RecursiveView"("X") AS SELECT 3 AS "X";
-- Comments and casing must not prevent recognition of the declaration prefix.
create /* prefix */ recursive /* modifier */ view app.commented(x) AS SELECT 4 AS x;
CREATE VIEW ordinary_view(x) AS SELECT 3 AS x;
CREATE OR REPLACE VIEW "App"."OrdinaryView"("X") AS SELECT 3 AS "X";
CREATE MATERIALIZED VIEW materialized_view(x) AS SELECT 3 AS x;
-- RECURSIVE outside a declaration modifier must keep its original meaning.
CREATE VIEW "recursive"(x) AS WITH RECURSIVE nums(x) AS (SELECT 1 UNION ALL SELECT x + 1 FROM nums WHERE x < 3) SELECT x FROM nums;
SELECT 'CREATE RECURSIVE VIEW', 1 AS "recursive";

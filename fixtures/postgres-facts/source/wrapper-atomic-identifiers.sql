-- BEGIN ATOMIC can be a column and alias pair without opening a nested body.
CREATE FUNCTION atomic_alias() RETURNS int BEGIN ATOMIC
  SELECT begin atomic FROM data;
  SELECT begin AS atomic FROM data;
  SELECT 1 AS case;
  SELECT 2 AS end;
  SELECT data.case FROM data;
  SELECT data.end FROM data;
  SELECT 1 case FROM data;
  SELECT 2 case;
  -- Function-name keywords can introduce a real simple CASE base expression.
  SELECT CASE LEFT('abc', 1) WHEN 'a' THEN 1 ELSE 0 END;
  SELECT 1 + CASE RIGHT('abc', 1) WHEN 'c' THEN 1 ELSE 0 END;
  SELECT 1 BETWEEN CASE LEFT('abc', 1) WHEN 'a' THEN 1 ELSE 0 END AND 2;
  SELECT 1 IS DISTINCT FROM CASE LEFT('abc', 1) WHEN 'a' THEN 1 ELSE 0 END;
  -- A bare table alias before a JOIN subquery must not become a CASE block.
  SELECT 1 FROM data case JOIN (SELECT 2) derived ON true;
  SELECT 1 FROM (SELECT 2) case JOIN (SELECT 3) derived ON true;
END;
SELECT 93;
CREATE OR REPLACE FUNCTION nested_alias() RETURNS void BEGIN ATOMIC
  CREATE OR REPLACE FUNCTION inner_alias() RETURNS int BEGIN ATOMIC
    SELECT begin atomic FROM data;
  END;
END;
SELECT 94;
-- Unsupported statements and malformed headers still preserve the outer END.
CREATE FUNCTION unsupported_children() RETURNS void BEGIN ATOMIC
  CREATE TABLE data_copy (id int);
  CREATE OR TABLE malformed;
  SELECT 1;
END;
SELECT 95;
-- Argument names and type names do not introduce a body inside its header.
CREATE FUNCTION argument_alias() RETURNS void BEGIN ATOMIC
  CREATE FUNCTION inner_arg(begin atomic) RETURNS TABLE(begin atomic) AS 'SELECT 1';
END;
SELECT 96;

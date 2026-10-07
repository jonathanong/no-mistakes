-- Native child parsing may reject a bare END label; it still belongs to this body.
CREATE FUNCTION end_projection() RETURNS int BEGIN ATOMIC
  SELECT 1 end FROM source;
  SELECT 2;
END;
SELECT 101;
CREATE FUNCTION end_comma() RETURNS int BEGIN ATOMIC
  SELECT 1 end, 2;
  SELECT 3;
END;
SELECT 102;
CREATE FUNCTION end_table_alias() RETURNS int BEGIN ATOMIC
  SELECT 1 FROM (SELECT 2) end JOIN source ON true;
  SELECT 4;
END;
SELECT 103;
-- Real CASE closers before those same tokens must still close expression frames.
CREATE FUNCTION real_end() RETURNS int BEGIN ATOMIC
  SELECT CASE WHEN true THEN 1 ELSE 0 END FROM source;
  SELECT CASE WHEN true THEN 1 ELSE 0 END, 2;
  SELECT CASE WHEN true THEN 1 ELSE 0 END AS end;
  SELECT CASE WHEN true THEN CASE WHEN false THEN 2 ELSE 1 END ELSE 0 END FROM source;
  SELECT source.end FROM source;
  SELECT 1 AS end FROM source;
END;
SELECT 104;
-- An alias inside a deeper query cannot consume an outer CASE frame.
CREATE FUNCTION end_nested_query() RETURNS int BEGIN ATOMIC
  SELECT CASE WHEN EXISTS(SELECT 1 end FROM source) THEN 1 ELSE 2 END;
  SELECT 5;
END;
SELECT 105;
-- A broken CASE child cannot claim the END after its original delimiter.
CREATE FUNCTION broken_case() RETURNS int BEGIN ATOMIC
  SELECT CASE WHEN true THEN 1;
  SELECT 6;
END;
SELECT 106;

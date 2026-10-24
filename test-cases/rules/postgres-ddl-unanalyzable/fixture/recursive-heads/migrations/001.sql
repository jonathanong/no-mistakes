-- Missing recursive-view columns are deliberately invalid; retain their original heads.
CREATE RECURSIVE VIEW top_level AS SELECT 1;
CREATE OR REPLACE TEMPORARY RECURSIVE VIEW replacement AS SELECT 1;
/* π */ CREATE RECURSIVE VIEW malformed (x) AS SELECT (;
DO $outer$
BEGIN;
  DO $inner$
  BEGIN;
    CREATE RECURSIVE VIEW nested AS SELECT 1;
  END
  $inner$;
END
$outer$;
-- These valid siblings must retain their recursive ASTs without gaining failures.
CREATE RECURSIVE VIEW valid (x) AS SELECT 1;
CREATE OR REPLACE TEMP RECURSIVE VIEW valid_temp (x) AS SELECT 1;
-- no-mistakes-disable-next-line postgres-fk-index: preserve normal line suppression.
CREATE RECURSIVE VIEW suppressed AS SELECT 1;
CREATE RECURSIVE VIEW; SELECT 1;
CREATE RECURSIVE VIEW table_body AS TABLE accounts;
DO $guard$
BEGIN;
  IF TRUE THEN CREATE RECURSIVE VIEW;
  END IF;
  IF TRUE THEN CREATE RECURSIVE VIEW guarded_columns AS SELECT 1;
  END IF;
END
$guard$;

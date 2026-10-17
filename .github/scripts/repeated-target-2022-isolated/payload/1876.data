-- Conditional partial targets remain an explicit pre-existing unsupported form.
DO $$BEGIN IF TRUE THEN INSERT INTO target VALUES (1) ON CONFLICT (id) WHERE id > 0 DO NOTHING; END IF; END$$;
SELECT 89;
DO $$BEGIN IF TRUE THEN EXPLAIN INSERT INTO target VALUES (1) ON CONFLICT (id) WHERE id > 0 DO NOTHING; END IF; END$$;
SELECT 90;
DO $$BEGIN IF TRUE THEN PREPARE conditional_partial AS WITH src AS (SELECT 1 AS id) INSERT INTO target SELECT id FROM src ON CONFLICT (id) WHERE id > 0 DO NOTHING; END IF; END$$;
SELECT 91;

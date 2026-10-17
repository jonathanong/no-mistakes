-- BEGIN/ATOMIC are expression/alias identifiers, not another atomic body.
CREATE FUNCTION conflict_alias() RETURNS void LANGUAGE SQL BEGIN ATOMIC
  INSERT INTO target SELECT begin atomic FROM data ON CONFLICT (id) WHERE id > 0 DO NOTHING;
END;
SELECT 111;
CREATE FUNCTION into_alias() RETURNS void LANGUAGE SQL BEGIN ATOMIC
  SELECT 1 case INTO tmp;
END;
SELECT 112;
EXPLAIN INSERT INTO target SELECT begin atomic FROM data ON CONFLICT (id) WHERE id > 0 DO NOTHING;
PREPARE alias_insert AS INSERT INTO target SELECT begin atomic FROM data ON CONFLICT (id) WHERE id > 0 DO NOTHING;
EXPLAIN (ANALYZE TRUE) SELECT 1 case INTO tmp;
PREPARE into_plan AS SELECT 1 case INTO tmp;

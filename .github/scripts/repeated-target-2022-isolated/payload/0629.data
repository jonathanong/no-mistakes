INSERT INTO t (id, v) VALUES (1, 2) ON CONFLICT (id) DO UPDATE SET v = COALESCE(t.v, EXCLUDED.v);
INSERT INTO t (id, v) VALUES (1, 2) ON CONFLICT (id) DO UPDATE SET v = GREATEST(t.v, EXCLUDED.v);
INSERT INTO t (id, v) VALUES (1, 2) ON CONFLICT (id) DO UPDATE SET v = EXCLUDED.v;
INSERT INTO t (id, v) VALUES (1, 2) ON CONFLICT (id) DO UPDATE SET v = (COALESCE(GREATEST(t.v, EXCLUDED.v), 0, $1))::integer;
-- Unknown lineage is distinct from missing syntax; this qualifier is intentionally unresolved.
INSERT INTO t (id, v) VALUES (1, 2) ON CONFLICT (id) DO UPDATE SET v = other.v;
INSERT INTO t AS excluded (id, v) VALUES (1, 2) ON CONFLICT (id) DO UPDATE SET v = excluded.v;
INSERT INTO t (id, v) VALUES (1, 2) ON CONFLICT (id) DO UPDATE SET v = clock_timestamp();
-- Wildcards and unprojected operand shapes must remain incomplete, even inside calls.
INSERT INTO t (id, v) VALUES (1, 2) ON CONFLICT (id) DO UPDATE SET v = count(*);
INSERT INTO t (id, v) VALUES (1, 2) ON CONFLICT (id) DO UPDATE SET v = COALESCE(t.v + EXCLUDED.v, 0);
INSERT INTO t (id, v) VALUES (1, 2) ON CONFLICT (id) DO UPDATE SET v = (SELECT v FROM other);
INSERT INTO t (id, v) VALUES (1, 2) ON CONFLICT (id) DO UPDATE SET v = COALESCE(t.v, );

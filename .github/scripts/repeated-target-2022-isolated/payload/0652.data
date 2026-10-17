-- Unary syntax exposes the operand; provenance alone must not bypass completeness.
INSERT INTO t VALUES (1) ON CONFLICT (id) DO UPDATE SET v = -1, n = +2;
INSERT INTO t VALUES (1) ON CONFLICT (id) DO UPDATE SET v = COALESCE(t.v, -1, +$1);
INSERT INTO t VALUES (1) ON CONFLICT (id) DO UPDATE SET v = -EXCLUDED.v;
INSERT INTO t VALUES (1) ON CONFLICT (id) DO UPDATE SET v = -(t.v + EXCLUDED.v);

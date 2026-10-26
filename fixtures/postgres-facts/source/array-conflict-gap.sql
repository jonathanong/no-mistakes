-- This malformed INSERT is counted lexically but cannot produce a complete AST.
INSERT INTO t (a[) VALUES (0);
INSERT INTO t VALUES (1) ON CONFLICT (id) DO UPDATE SET a[1] = EXCLUDED.a[1];
INSERT INTO t(id, a[1:2]) VALUES (2, ARRAY[1,2]) ON CONFLICT (id)
  DO UPDATE SET a[1] = EXCLUDED.a[1];
INSERT INTO t VALUES (4);
INSERT INTO t (id, a) OVERRIDING USER VALUE VALUES (5, ARRAY[1,2]);

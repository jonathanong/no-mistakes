INSERT INTO t SELECT coalesce(f[1], 0) FROM (VALUES (1)) AS v(x);
INSERT INTO t (a) SELECT coalesce(f[1], 0) FROM (VALUES (1)) AS v(x);
UPDATE op SET x = arr[1] = 3;
UPDATE op SET f1[2] = 3, x = arr[1] = 3;

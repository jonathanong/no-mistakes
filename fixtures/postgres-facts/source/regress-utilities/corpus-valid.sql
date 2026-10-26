-- PostgreSQL 17 regression witnesses: horology, subselect, and create_role.
EXPLAIN (COSTS OFF) SELECT count(*) FROM date_tbl WHERE f1 BETWEEN SYMMETRIC '1997-01-01' AND '1998-01-01';
DECLARE c1 SCROLL CURSOR FOR SELECT * FROM generate_series(1,4) i WHERE i <> ALL (VALUES (2),(3));
GRANT regress_tenant2 TO regress_createrole WITH INHERIT TRUE, SET FALSE;

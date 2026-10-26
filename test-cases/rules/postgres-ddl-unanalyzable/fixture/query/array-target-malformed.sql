insert into arr_pk_tbl(pk, f1[]) values (1, '{6,7,8}') on conflict (pk) do nothing;
insert into arr_pk_tbl(pk, f1[1:2]) values (1, '{6,7,8}') on conflict (pk) do update set f1[1] =;
update op set f1[] = 3;
UPDATE arrtest_s SET a[::] = '{1}';
update op set f1[2] =;
insert into itest1 overriding user value;
insert into itest1 overriding user values (1);
INSERT INTO t (SELECT[1]) VALUES (1);
UPDATE SELECT SET a[1] = 1;
-- PostgreSQL rejects a third slice component even though sqlparser accepts it.
INSERT INTO t (a[1:2:3]) VALUES (ARRAY[1,2]);
UPDATE t SET a[1:2:3] = ARRAY[1,2];

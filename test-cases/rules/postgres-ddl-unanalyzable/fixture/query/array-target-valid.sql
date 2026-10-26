-- Array-slice targets must retain the ON CONFLICT AST, these are PostgreSQL regress witnesses.
insert into arr_pk_tbl values (1, '{3,4,5}') on conflict (pk)
  do update set f1[1] = excluded.f1[1], f1[3] = excluded.f1[3]
  returning pk, f1;
insert into arr_pk_tbl(pk, f1[1:2]) values (1, '{6,7,8}') on conflict (pk)
  do update set f1[1] = excluded.f1[1],
    f1[2] = excluded.f1[2],
    f1[3] = excluded.f1[3]
  returning pk, f1;
insert into t1 (f1[5].q1) values(42) on conflict do nothing;
update op set f1[2] = 3;
-- PostgreSQL permits an omitted lower and upper slice bound.
UPDATE arrtest_s SET a[:] = '{11, 12, 13, 14, 15}';
insert into itest1 overriding user value values (30, 'xyz') on conflict do nothing;
insert into itest5 overriding system value values (1, 'a') on conflict do nothing;
-- Identity overrides without an arbiter clause remain INSERT facts.
insert into itest1 overriding user value values (30, 'xyz');
insert into itest5 overriding system value values (1, 'a');

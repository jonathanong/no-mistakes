-- test indexes on arrays
create temp table arr_tbl (f1 int[] unique);
insert into arr_tbl values ('{1,2,3}');
insert into arr_tbl values ('{1,2}');
-- failure expected:
insert into arr_tbl values ('{1,2,3}');
insert into arr_tbl values ('{2,3,4}');
insert into arr_tbl values ('{1,5,3}');
insert into arr_tbl values ('{1,2,10}');

set enable_seqscan to off;
set enable_bitmapscan to off;
select * from arr_tbl where f1 > '{1,2,3}' and f1 <= '{1,5,3}';
select * from arr_tbl where f1 >= '{1,2,3}' and f1 < '{1,5,3}';

-- test ON CONFLICT DO UPDATE with arrays
create temp table arr_pk_tbl (pk int4 primary key, f1 int[]);
insert into arr_pk_tbl values (1, '{1,2,3}');
insert into arr_pk_tbl values (1, '{3,4,5}') on conflict (pk)
  do update set f1[1] = excluded.f1[1], f1[3] = excluded.f1[3]
  returning pk, f1;
insert into arr_pk_tbl(pk, f1[1:2]) values (1, '{6,7,8}') on conflict (pk)
  do update set f1[1] = excluded.f1[1],
    f1[2] = excluded.f1[2],
    f1[3] = excluded.f1[3]
  returning pk, f1;

-- note: if above selects don't produce the expected tuple order,
-- then you didn't get an indexscan plan, and something is busted.
reset enable_seqscan;

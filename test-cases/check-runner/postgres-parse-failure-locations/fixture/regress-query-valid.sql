SELECT * FROM a_star*;
SELECT collation for ('foo');
SELECT JSON_VALUE(jsonb '123', '$' RETURNING int);
SELECT JSON_QUERY(js, '$' RETURNING int DEFAULT b + 1 ON ERROR) FROM test_jsonb_mutability;
SELECT JSON_EXISTS(jsonb '{"a": 1, "b": 2}', '$.* ? (@ > $x)' PASSING 1 AS x);
SELECT JSON_EXISTS(jsonb '{"a": 1}', '$' PASSING 1 AS x, 2 AS y);
SELECT JSON_ARRAY(RETURNING json);
SELECT JSON_OBJECT(1: 1, '1': NULL WITH UNIQUE);
SELECT JSON_OBJECT(1: 1, '1': NULL ABSENT ON NULL WITH UNIQUE RETURNING jsonb);
SELECT * FROM JSON_TABLE (NULL::jsonb, '$' COLUMNS (v1 timestamp)) AS f (v1, v2);
SELECT xmlpi(name foo);
SELECT xmlserialize(document 'bad' as text);
SELECT xml '<foo>bar</foo>' IS DOCUMENT;
SELECT * FROM XMLTABLE('/ROWS/ROW' PASSING data COLUMNS id int PATH '@id');
select sum(salary) over (order by enroll_date range between '1 year'::interval preceding and '1 year'::interval following) from empsalary;
SELECT count(*) FROM test_tablesample TABLESAMPLE SYSTEM (50);
select * from rows from(unnest(array[10,20])) with ordinality as z(a,ord);
with recursive x(col) as (select 1 union all select col + 1 from x where col < 3) select * from x;
WITH merge_source_cte AS MATERIALIZED (SELECT 15 a, 'v' b) SELECT * FROM merge_source_cte;
WITH foo AS (
  MERGE INTO target USING source ON (true)
  WHEN MATCHED THEN DELETE
) SELECT * FROM foo;
insert into t1 (f1[5].q1) values(42);
INSERT INTO itest1 OVERRIDING USER VALUE VALUES (30, 'xyz');
INSERT INTO itest5 OVERRIDING SYSTEM VALUE VALUES (1, 'a');
update op set f1[2] = 3;
VALUES (TIMESTAMP '1978-07-07 19:38' AT LOCAL);
SELECT xmlserialize(document 'bad' as "select");
SELECT public.from, xmlserialize(document 'bad' as text) FROM t;
SELECT xmlserialize(document 'bad' as text) FROM t AS x;
SELECT * FROM JSON_TABLE (NULL::jsonb, '$' COLUMNS ("SELECT" "select")) AS f;
SELECT * FROM rows from(unnest(array[1])) with ordinality as z(a int, b int);
insert into arr_pk_tbl values (1, '{3,4,5}') on conflict (pk)
  do update set f1[1] = excluded.f1[1], f1[3] = excluded.f1[3]
  returning pk, f1;
insert into arr_pk_tbl(pk, f1[1:2]) values (1, '{6,7,8}') on conflict (pk)
  do update set f1[1] = excluded.f1[1],
    f1[2] = excluded.f1[2],
    f1[3] = excluded.f1[3]
  returning pk, f1;

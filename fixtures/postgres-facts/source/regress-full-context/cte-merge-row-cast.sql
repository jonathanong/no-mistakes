WITH merge_source_cte AS MATERIALIZED (SELECT 15 a, 'merge_source_cte val' b)
MERGE INTO m USING (select * from merge_source_cte) o ON m.k=o.a
WHEN MATCHED THEN UPDATE SET v = (SELECT b || merge_source_cte.*::text || ' merge update' FROM merge_source_cte WHERE a = 15)
WHEN NOT MATCHED THEN INSERT VALUES(o.a, o.b || (SELECT merge_source_cte.*::text || ' merge insert' FROM merge_source_cte));

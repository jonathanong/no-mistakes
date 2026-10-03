SELECT 'λ' AS note LIMIT 0xF_F;
-- no-mistakes-disable-next-line postgres-sql-shape-policy
SELECT 1 LIMIT 0xF_F;
SELECT 1 LIMIT X'FF';
SELECT 1 LIMIT x'FF';
SELECT 1 LIMIT 0x0;
-- no-mistakes-disable-next-line postgres-sql-shape-policy
SELECT 1 LIMIT 0x1;

SELECT 1 FROM notices AS n
UNION
SELECT 1 FROM tasks AS t
FOR UPDATE OF n;

SELECT 1 FROM (SELECT 1) AS derived
FOR UPDATE OF derived;

SELECT 1 FROM generate_series(1, 2) AS generated(id)
FOR UPDATE OF generated;

SELECT 1 FROM public.notices
FOR UPDATE OF public.notices;

SELECT 1 FROM notices
FOR UPDATE OF notices;

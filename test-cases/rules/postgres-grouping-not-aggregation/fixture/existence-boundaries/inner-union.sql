SELECT e.tenant FROM events e GROUP BY e.tenant HAVING NOT EXISTS (SELECT 1 FROM events i WHERE i.tenant = e.tenant UNION ALL SELECT 2) OR MAX(e.score) < 10;

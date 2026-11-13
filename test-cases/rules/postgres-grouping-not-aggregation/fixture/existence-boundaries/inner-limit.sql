SELECT e.tenant FROM events e GROUP BY e.tenant HAVING NOT EXISTS (SELECT 1 FROM events i WHERE i.tenant = e.tenant LIMIT 1) OR MAX(e.score) < 10;

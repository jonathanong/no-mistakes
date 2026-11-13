SELECT e.tenant FROM (SELECT tenant, score FROM events) e GROUP BY e.tenant HAVING NOT EXISTS (SELECT 1 FROM events i WHERE i.tenant = e.tenant) OR MAX(e.score) < 10;

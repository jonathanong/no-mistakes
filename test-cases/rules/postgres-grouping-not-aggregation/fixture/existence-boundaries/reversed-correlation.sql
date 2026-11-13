SELECT e.tenant FROM events e GROUP BY e.tenant HAVING NOT EXISTS (SELECT 1 FROM events i WHERE e.tenant = i.tenant) OR MAX(e.score) < 10;

SELECT e.tenant, COUNT(*) FROM events e GROUP BY e.tenant HAVING NOT EXISTS (SELECT 1 FROM events i WHERE i.tenant = e.tenant) OR MAX(e.score) < 10;

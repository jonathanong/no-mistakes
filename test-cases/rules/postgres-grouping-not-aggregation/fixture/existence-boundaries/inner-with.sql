SELECT e.tenant FROM events e GROUP BY e.tenant HAVING NOT EXISTS (WITH helper AS (SELECT 1) SELECT 1 FROM events i WHERE i.tenant = e.tenant) OR MAX(e.score) < 10;

SELECT e.tenant FROM events e GROUP BY e.tenant HAVING MAX(e.score) < 10 OR NOT EXISTS (SELECT 1 FROM events i WHERE i.tenant = e.tenant);

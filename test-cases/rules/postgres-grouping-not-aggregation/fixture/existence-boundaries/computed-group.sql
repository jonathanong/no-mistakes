SELECT e.tenant + 0 FROM events e GROUP BY e.tenant + 0 HAVING NOT EXISTS (SELECT 1 FROM events i WHERE i.tenant = e.tenant) OR MAX(e.score) < 10;

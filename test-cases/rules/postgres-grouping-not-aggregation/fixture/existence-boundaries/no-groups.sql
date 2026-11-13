SELECT 1 FROM events e HAVING NOT EXISTS (SELECT 1 FROM events i WHERE i.tenant = e.tenant) OR MAX(e.score) < 10;

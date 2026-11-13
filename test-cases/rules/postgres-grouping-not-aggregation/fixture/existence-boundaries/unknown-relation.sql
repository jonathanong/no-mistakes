SELECT e.tenant FROM missing_relation e GROUP BY e.tenant HAVING NOT EXISTS (SELECT 1 FROM missing_relation i WHERE i.tenant = e.tenant) OR MAX(e.score) < 10;

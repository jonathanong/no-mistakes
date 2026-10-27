UPDATE accounts a SET name = 'x' FROM (VALUES ($1), ($2)) AS v(id) WHERE a.id = v.id;

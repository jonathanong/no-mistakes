UPDATE accounts a SET name = 'x' FROM unnest($1::uuid[]) AS ids(id) WHERE a.id = ids.id;

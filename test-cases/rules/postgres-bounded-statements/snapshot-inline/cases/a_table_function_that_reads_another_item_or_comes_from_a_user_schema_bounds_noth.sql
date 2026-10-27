SELECT * FROM accounts a JOIN unnest(ARRAY[a.id]) u(id) ON a.id = u.id;

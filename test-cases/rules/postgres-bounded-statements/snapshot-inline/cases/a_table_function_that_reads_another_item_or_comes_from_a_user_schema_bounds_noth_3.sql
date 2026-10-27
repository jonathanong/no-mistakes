UPDATE accounts a SET name = 'x' FROM app.generate_series(1, 10) g(id) WHERE a.id = g.id;

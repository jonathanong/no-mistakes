CREATE VIEW app.called AS SELECT pg_catalog.lower(a.name), app.decorate(a.name) FROM app.accounts a WHERE app.allowed(a.id) OR app.allowed(a.id);

-- Same-named application functions do not inherit pg_catalog SRF identity.
SELECT app.pg_ls_dir(count(*)) FROM orders;
SELECT pg_catalog.pg_ls_dir(count(*)) FROM orders;
SELECT pg_ls_dir(count(*)) FROM orders;

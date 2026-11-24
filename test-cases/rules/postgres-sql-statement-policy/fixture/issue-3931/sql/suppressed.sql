-- no-mistakes-disable-next-line postgres-sql-statement-policy
DROP EXTENSION widgets CASCADE;
DROP SEQUENCE widgets.id_seq CASCADE; -- no-mistakes-disable-line postgres-sql-statement-policy
DROP DOMAIN widgets.label CASCADE;

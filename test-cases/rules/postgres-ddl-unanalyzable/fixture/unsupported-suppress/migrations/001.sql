-- no-mistakes-disable-next-line postgres-fk-index: reviewed unsupported utility
CREATE PUBLICATION app_changes FOR ALL TABLES;
REFRESH MATERIALIZED VIEW account_totals; -- no-mistakes-disable-line postgres-fk-index: reviewed refresh
REFRESH MATERIALIZED VIEW account_totals;

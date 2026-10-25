-- no-mistakes-disable-file postgres-fk-index: reviewed unsupported utilities
CREATE PUBLICATION app_changes FOR ALL TABLES;
REFRESH MATERIALIZED VIEW account_totals;

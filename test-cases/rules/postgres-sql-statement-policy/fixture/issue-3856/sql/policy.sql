CREATE EXTENSION IF NOT EXISTS widgets VERSION '1.4';
DROP VIEW widget_summary CASCADE;
DROP MATERIALIZED VIEW widget_snapshot CASCADE;
DROP TABLE widgets CASCADE;
DROP FUNCTION summarize_widgets() CASCADE;
DROP TYPE widget_kind CASCADE;
DROP SCHEMA widgets_archive CASCADE;
DROP INDEX widgets_lookup CASCADE;
ALTER TABLE widgets DROP COLUMN description CASCADE;
TRUNCATE TABLE widgets CASCADE;

CREATE EXTENSION IF NOT EXISTS versioned_widgets;
CREATE EXTENSION "VERSION";
DROP VIEW "CASCADE" RESTRICT;
DROP TABLE widgets_archive.widgets;
ALTER TABLE widgets ADD FOREIGN KEY (source_id) REFERENCES sources (id) ON DELETE CASCADE;
SELECT 'DROP VIEW widget_summary CASCADE';
-- no-mistakes-disable-next-line postgres-sql-statement-policy
DROP VIEW replaced_widget_summary CASCADE;

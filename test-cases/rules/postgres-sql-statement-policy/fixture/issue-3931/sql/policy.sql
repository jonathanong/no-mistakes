DROP EXTENSION IF EXISTS widgets, widget_tools CASCADE;
DROP SEQUENCE IF EXISTS widgets.id_seq, widgets.audit_seq CASCADE;
DROP DOMAIN IF EXISTS widgets.label CASCADE;
DROP TRIGGER IF EXISTS audit_change ON widgets.records CASCADE;
ALTER TABLE widgets.records
  DROP CONSTRAINT IF EXISTS unique_label CASCADE;
ALTER TABLE widgets.archive
  DROP CONSTRAINT IF EXISTS unique_label CASCADE,
  DROP COLUMN IF EXISTS previous_label CASCADE;

DROP EXTENSION widgets RESTRICT;
DROP SEQUENCE widgets.id_seq RESTRICT;
DROP DOMAIN widgets.label RESTRICT;
DROP TRIGGER audit_change ON widgets.records RESTRICT;
ALTER TABLE widgets.records DROP CONSTRAINT unique_label RESTRICT;
ALTER TABLE widgets.records DROP CONSTRAINT "CASCADE";
CREATE TABLE widgets.records (id integer);
CREATE EXTENSION widgets CASCADE;

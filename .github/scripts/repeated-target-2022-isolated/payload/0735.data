-- Delimiter padding must not consume the neighboring statement or alter names.
CREATE VIEW view_topics AS SELECT id FROM safe UNION ALL TABLE "Topics";
CREATE TABLE after_table_arm (id integer);
SELECT id FROM safe UNION ALL TABLE topics;
ALTER TABLE after_table_arm ADD COLUMN note text;

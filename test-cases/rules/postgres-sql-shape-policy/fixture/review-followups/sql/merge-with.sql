WITH source AS (SELECT id FROM seed WHERE id NOT IN (SELECT id FROM bans))
MERGE INTO items AS target USING source
ON target.id = source.id
WHEN MATCHED AND target.id NOT IN (SELECT id FROM bans) THEN DELETE;

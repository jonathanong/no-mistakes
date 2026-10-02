-- Cross-dialect AST exposes mutation target joins that PostgreSQL normally expresses with FROM.
UPDATE tags JOIN orders o ON tags.id = o.id SET tags.name = 'paid' RETURNING o.*;

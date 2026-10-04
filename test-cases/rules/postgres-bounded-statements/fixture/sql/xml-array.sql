-- XML is parsed as a custom type but is a trusted PostgreSQL scalar builtin.
DELETE FROM accounts WHERE email = ANY(ARRAY[XML '<x/>']::text[]);
DELETE FROM accounts WHERE email = ANY(ARRAY[XML '<x/>', XML '<y/>']::text[]);
-- Qualified application types cannot inherit the builtin XML scalar proof.
DELETE FROM accounts WHERE email = ANY(ARRAY['<x/>'::app.xml]::text[]);
DELETE FROM accounts WHERE email = ANY(ARRAY['{x,y}'::app.array_value]::text[]);
-- Typed array literals can flatten multiple values and need separate evidence.
DELETE FROM accounts WHERE email = ANY(ARRAY[TEXT[] '{x,y}']);

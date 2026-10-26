-- The rule judges the JSONB expression; assignment coercion to text is not an exemption.
INSERT INTO orders (note) VALUES (jsonb_build_object('accountId', $1));

-- Literal delimiters must retain escaping, so a value cannot become SQL syntax.
SELECT * FROM accounts WHERE id > $1 AND status = 'a'' or x = ''b' ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE id > $1 AND (status = 'a' OR x = 'b') ORDER BY id LIMIT $2;
-- Quoted identifier delimiters have the same boundary invariant.
SELECT * FROM accounts WHERE id > $1 AND "a"" or x = ""b" ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE id > $1 AND ("a" OR x = "b") ORDER BY id LIMIT $2;
-- Embedded literal whitespace remains significant while outside whitespace normalizes.
SELECT * FROM accounts WHERE id > $1 AND status = 'a  b' ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE id > $1 AND status = 'a b' ORDER BY id LIMIT $2;
-- Escape-string values must never become SQL syntax during normalization.
SELECT * FROM accounts WHERE id > $1 AND status = E'a\' or x = \'b' ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE id > $1 AND (status = E'a' OR x = 'b') ORDER BY id LIMIT $2;

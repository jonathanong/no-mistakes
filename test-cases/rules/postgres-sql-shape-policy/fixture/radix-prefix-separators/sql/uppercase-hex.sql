-- Uppercase prefixes tokenize as adjacent zero/word tokens, never implicit aliases.
'SELECT * FROM orders LIMIT ' || chr(48) || 'XFF';
'SELECT * FROM orders LIMIT ' || '0XF_F';
SELECT * FROM orders LIMIT 0XFF;
SELECT 0XFF;
SELECT * FROM orders LIMIT 0XFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
SELECT 0 XFF;
SELECT 0 AS "XFF";
SELECT * FROM orders LIMIT X'FF';

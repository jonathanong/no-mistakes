SELECT id FROM accounts WHERE COALESCE(sql_placeholder_1, token) = id;
SELECT id FROM accounts WHERE COALESCE("sql_placeholder_1", token) = id;

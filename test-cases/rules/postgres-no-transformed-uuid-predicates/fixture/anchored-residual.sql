SELECT * FROM accounts a WHERE a.id = $1 AND fn_remaining_duration(a.token) > interval '0';
SELECT * FROM accounts a WHERE a.id = ANY($1::uuid[]) AND fn_remaining_duration(a.token) > interval '0';
SELECT * FROM accounts a WHERE fn_remaining_duration(a.token) > interval '0';

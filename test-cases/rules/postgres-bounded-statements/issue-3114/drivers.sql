SELECT a.id FROM accounts a JOIN profiles p ON p.account_id = a.id WHERE a.id = $1;
SELECT a.id FROM accounts a JOIN profiles p ON p.account_id = a.id WHERE a.id = ANY($1::uuid[]);
SELECT a.id FROM accounts a JOIN contacts c ON c.email = a.email WHERE a.id = $1;
SELECT a.id FROM accounts a JOIN profiles p ON p.account_id = a.id WHERE a.name = $1;

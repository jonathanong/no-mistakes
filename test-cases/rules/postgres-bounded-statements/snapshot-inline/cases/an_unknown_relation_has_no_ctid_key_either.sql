UPDATE accounts a SET name = 'x' FROM external_events e WHERE e.ctid = $1 AND a.id = e.account_id;

WITH expanded AS (
  SELECT r.id FROM accounts a CROSS JOIN get_ids(a.id) AS r(id)
)
UPDATE profiles p SET bio = 'done' FROM expanded s WHERE p.account_id = s.id;

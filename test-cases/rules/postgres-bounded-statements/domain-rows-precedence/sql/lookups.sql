SELECT code FROM currencies WHERE code > $1 ORDER BY code LIMIT $2;
SELECT id FROM ledger_rows WHERE batch_id = $1 AND id > $2 ORDER BY id LIMIT $3;
SELECT note FROM mixed WHERE batch_id = $1;
SELECT note FROM mixed;
SELECT body FROM notes;

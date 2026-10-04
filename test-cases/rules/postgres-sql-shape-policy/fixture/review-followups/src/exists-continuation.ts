import { query } from "@example/db";

const statement = `SELECT id FROM accounts WHERE \
EXISTS (
  SELECT 1 FROM events WHERE events.account_id = accounts.id
  UNION ALL
  SELECT 1 FROM archived WHERE archived.account_id = accounts.id
)`;
query(statement);

import { query } from "db";
query("SELECT 1 FROM accounts WHERE id::text = $1");
const statement = "SELECT 1 FROM accounts WHERE CAST(id AS text) = $1";
query(statement);
query(`SELECT 1 FROM accounts WHERE uuid_extract_timestamp(id) > $1`);
query("SELECT 1 FROM accounts WHERE id = $1::uuid");

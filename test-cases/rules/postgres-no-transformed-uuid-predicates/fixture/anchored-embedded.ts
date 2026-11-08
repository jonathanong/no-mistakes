import { query } from "db";
query(`SELECT * FROM accounts WHERE id = $1 AND fn(token) = $2`);
query(`SELECT * FROM accounts WHERE fn(token) = $1`);
const statement = "SELECT * FROM accounts WHERE id = $1 AND fn(token) = $2";
query(statement);
query(`SELECT * FROM accounts WHERE id = ANY($1::uuid[]) AND fn(token) = $2`);
query(`SELECT * FROM accounts a LEFT JOIN accounts b ON b.id = a.token WHERE fn(b.token) = $1`);
query(`SELECT * FROM accounts WHERE id >= $1 AND id < $2 AND uuid_extract_timestamp(id) >= $3`);
query(`SELECT * FROM accounts WHERE id = $1 OR fn(token) = $2`);

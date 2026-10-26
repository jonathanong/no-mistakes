import { Pool, Client } from "pg";

// Imported constructors do not prove query identity on their instances.
const pool = new Pool();
const connected = await pool.connect();
const client = new Client();
for (const executor of [pool, connected, client]) {
  executor.query("BEGIN");
}
pool.query("COMMIT");
connected.query("ROLLBACK");
client.query("START TRANSACTION");
Promise.all(ids.map((id) => pool.query(`SELECT ${id}`)));
Promise.all(ids.map((id) => connected.query(`SELECT ${id}`)));
Promise.all(ids.map((id) => client.query(`SELECT ${id}`)));
pool.query(`SELECT * FROM ${registry[key]}`);
connected.query(`SELECT * FROM ${registry[key]}`);
client.query(`SELECT * FROM ${registry[key]}`);

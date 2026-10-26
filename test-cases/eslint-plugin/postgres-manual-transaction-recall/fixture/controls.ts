import { writePool } from "@data-stores/psql";
const client = await writePool.connect();
await client.query("SELECT 1");
await client.query("UPDATE users SET active = true");

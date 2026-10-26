import { Pool } from "pg";
const writePool = new Pool();
const direct = await writePool.connect();
await direct.query("BEGIN");
await direct.query("ROLLBACK");
const connect = writePool.connect;
const reflected = await Reflect.apply(connect, writePool, []);
await reflected.query("BEGIN");
const called = await connect.call(writePool);
await called.query("START TRANSACTION");
const applied = await connect.apply(writePool, []);
await applied.query("COMMIT");
const boundConnect = connect.bind(writePool);
const bound = await boundConnect();
await bound.query("ROLLBACK");
// Manual transaction recall must survive a receiver the analyzer cannot prove.
const unknown = getClient();
await unknown.query("BEGIN");
await unknown.query("SELECT 1");
// no-mistakes-disable-next-line postgres-no-manual-transaction
await unknown.query("ROLLBACK");

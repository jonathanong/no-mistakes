import { beginTransaction } from "db";
import sql from "sql-template-strings";
import { getConfig, getFriendConfig, getProviderConfig } from "./config.mts";

export async function write(dynamicArg: string, kind: string, provider: string) {
  const config = getConfig(dynamicArg);
  const friend = getFriendConfig(kind);
  const providerConfig = getProviderConfig(provider);
  const query = await beginTransaction();
  await query(sql`/* friend */ DELETE FROM ${friend.table} WHERE ${friend.column} = $1`);
  await query(sql`/* provider */ INSERT INTO ${providerConfig.retainedTable} (id) VALUES ($1)`);
  await query(sql`/* table */ INSERT INTO ${config.retainedTable} (id) VALUES ($1)`);
  await query(
    sql`/* name */ INSERT INTO impacts (request_id, subject_id, ${config.integrityTargetColumn}) VALUES ($1, $2, $3) RETURNING id`,
  );
}

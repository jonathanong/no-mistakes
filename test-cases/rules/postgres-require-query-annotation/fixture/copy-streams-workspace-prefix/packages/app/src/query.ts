import { from as copyFrom, to as copyTo } from 'pg-copy-streams';
import type { PoolClient } from '@pkg/psql';

export async function stage(client: PoolClient, table: string) {
  const copyStream = client.query(copyFrom(`/* syncDomains */ COPY ${table} (domain) FROM STDIN`));
  return copyStream;
}

function orderSql(idLiteral: string) {
  return `SELECT domain FROM memberships WHERE id = ${idLiteral}`;
}

export async function copyOut(client: PoolClient, idLiteral: string) {
  const copy = client.query(copyTo(`/* copyMembership */
    COPY (
      ${orderSql(idLiteral)}
    ) TO STDOUT`));
  return copy;
}

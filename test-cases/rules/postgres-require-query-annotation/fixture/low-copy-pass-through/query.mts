import type { PoolClient } from 'pg';
import { from as copyFrom, to as copyTo } from 'pg-copy-streams';
import { to as unconfiguredCopy } from 'other-copy-streams';

export function run(client: PoolClient, dynamic: string) {
  client.query(copyFrom('/* import users */ COPY users FROM STDIN'));
  client.query(copyTo(`/* export users */ COPY users TO STDOUT`));
  client.query(copyFrom('COPY users FROM STDIN'));
  client.query(copyTo('COPY users TO STDOUT'));
  client.query(copyFrom(dynamic));
  client.query(unconfiguredCopy('/* export users */ COPY users TO STDOUT'));
  client.query(copyTo?.('/* export users */ COPY users TO STDOUT'));
}

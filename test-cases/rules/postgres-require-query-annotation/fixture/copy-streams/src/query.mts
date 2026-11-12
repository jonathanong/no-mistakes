import type { PoolClient } from 'pg';
import { from, to, from as copyFrom, to as copyTo } from 'pg-copy-streams';
import * as copies from 'pg-copy-streams';
import { from as otherFrom, to as otherTo } from 'other-copy-streams';
import type { from as typeOnly } from 'pg-copy-streams';

export function run(client: PoolClient, dynamic: string) {
  client.query(from('/* named from */ COPY users FROM STDIN'));
  client.query(to('/* named to */ COPY users TO STDOUT'));
  client.query(copyFrom(`/* alias from */ COPY users FROM STDIN`));
  client.query(copyTo(`/* alias to */ COPY users TO STDOUT`));
  client.query(copies.from('/* namespace from */ COPY users FROM STDIN'));
  client.query(copies.to('/* namespace to */ COPY users TO STDOUT'));
  client.query(copies['from']('/* computed from */ COPY users FROM STDIN'));
  client.query(copies[`to`]('/* computed to */ COPY users TO STDOUT'));
  client.query(from('COPY users FROM STDIN'));
  client.query(to(`COPY users TO STDOUT`));
  client.query(copyFrom('COPY users FROM STDIN'));
  client.query(copyTo('COPY users TO STDOUT'));
  client.query(copies.from('COPY users FROM STDIN'));
  client.query(copies.to('COPY users TO STDOUT'));
  client.query(copyFrom(/* outside SQL */ 'COPY users FROM STDIN'));
  client.query(copyFrom(dynamic));
  client.query(copyTo(`/* dynamic */ COPY ${dynamic} TO STDOUT`));
  client.query(copies.from(dynamic));
  client.query(copies.to(dynamic));
  client.query(otherFrom('/* unrelated */ COPY users FROM STDIN'));
  client.query(otherTo('/* unrelated */ COPY users TO STDOUT'));
  client.query(typeOnly('/* type only */ COPY users FROM STDIN'));
  client.query(copies?.from('/* optional */ COPY users FROM STDIN'));
  client.query(copyTo?.('/* optional */ COPY users TO STDOUT'));
  client.query(copies[dynamic]('/* dynamic key */ COPY users FROM STDIN'));
  {
    const from = (_: string) => dynamic;
    const to = (_: string) => dynamic;
    client.query(from('/* local from */ COPY users FROM STDIN'));
    client.query(to('/* local to */ COPY users TO STDOUT'));
  }
  {
    const copies = { from: (_: string) => dynamic, to: (_: string) => dynamic };
    client.query(copies.from('/* local namespace */ COPY users FROM STDIN'));
    client.query(copies.to('/* local namespace */ COPY users TO STDOUT'));
  }
}

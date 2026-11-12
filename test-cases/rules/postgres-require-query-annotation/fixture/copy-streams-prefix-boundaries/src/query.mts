import type { PoolClient } from 'pg';
import { from as copyFrom, to as copyTo } from 'pg-copy-streams';
import * as copies from 'pg-copy-streams';
import { from as otherFrom } from 'other-copy-streams';

export function run(client: PoolClient, dynamic: string) {
  client.query(copyFrom(`/* named */ COPY ${dynamic} FROM STDIN`));
  client.query(copyTo(`/* named */${dynamic}`));
  client.query(copies.from(`/* named */ COPY ${dynamic} FROM STDIN`));
  client.query(copies['to'](`/* named */ COPY ${dynamic} TO STDOUT`));
  client.query(copies[`from`](`/* named */ COPY ${dynamic} FROM STDIN`));
  client.query(copyFrom(`/* ${dynamic} */ COPY users FROM STDIN`));
  client.query(copyFrom(`/* named *${dynamic}/ COPY users FROM STDIN`));
  client.query(copyTo(`${dynamic} /* named */ COPY users TO STDOUT`));
  client.query(copyTo(`   ${dynamic} COPY users TO STDOUT`));
  client.query(copyFrom(`/* */ COPY ${dynamic} FROM STDIN`));
  client.query(copyFrom(`/* */ ${dynamic}`));
  client.query(copies[`from${dynamic}`](`/* named */ COPY users FROM STDIN`));
  client.query(otherFrom(`/* named */ COPY ${dynamic} FROM STDIN`));
  client.query(copyFrom(dynamic));
  client.query(copyFrom?.(`/* named */ COPY ${dynamic} FROM STDIN`));
  client.query(copyFrom(`COPY ${dynamic} FROM STDIN`));
}

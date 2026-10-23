import type { PoolClient } from 'pg';
import { from, from as copyFrom } from 'pg-copy-streams';
import * as copies from 'pg-copy-streams';
import { from as otherFrom } from 'other-copy-streams';
import type { from as typeOnly } from 'pg-copy-streams';

export function run(client: PoolClient, dynamic: string) {
  client.query(copyFrom('/* named */ COPY users FROM STDIN')); // clear
  client.query(copyFrom(`/* named */ COPY users FROM STDIN`)); // clear
  client.query(copies.from('/* named */ COPY users FROM STDIN')); // clear
  client.query(from('/* named */ COPY users FROM STDIN')); // clear
  client.query(copyFrom('BEGIN')); // clear
  client.query(copyFrom('COPY users FROM STDIN')); // missing
  // The JavaScript comment is not part of the SQL string.
  client.query(copyFrom(/* named */ 'COPY users FROM STDIN')); // missing
  client.query(copyFrom(`COPY users FROM STDIN`)); // missing
  client.query(copies['from']('/* named */ COPY users FROM STDIN')); // unknown
  client.query(copyFrom(42)); // unknown
  client.query(copyFrom('/* named */ ' + dynamic)); // unknown
  client.query(copyFrom(copies.from('/* named */ COPY users FROM STDIN'))); // unknown
  client.query(copies?.from('/* named */ COPY users FROM STDIN')); // unknown
  client.query(copyFrom()); // unknown
  client.query(copyFrom('/* named */ COPY users FROM STDIN', {})); // unknown
  client.query(copyFrom(dynamic)); // unknown
  client.query(copyFrom(`/* named */ COPY ${dynamic} FROM STDIN`)); // unknown
  client.query(copyFrom(...['/* named */ COPY users FROM STDIN'])); // unknown
  client.query(copyFrom?.('/* named */ COPY users FROM STDIN')); // unknown
  client.query(copies.from?.('/* named */ COPY users FROM STDIN')); // unknown
  client.query(otherFrom('/* named */ COPY users FROM STDIN')); // unknown
  client.query(typeOnly('/* named */ COPY users FROM STDIN')); // unknown
  const saved = copyFrom('/* named */ COPY users FROM STDIN');
  client.query(saved); // unknown
  { const copyFrom = (_: string) => dynamic;
    client.query(copyFrom('/* named */ COPY users FROM STDIN')); // unknown
  }
  { const copies = { from: (_: string) => dynamic };
    client.query(copies.from('/* named */ COPY users FROM STDIN')); // unknown
  }
}

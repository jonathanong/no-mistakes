import * as db from '@example/db';
await db.read?.('/* fetch */ SELECT 1');
await db['read']('/* fetch_again */ SELECT 2');

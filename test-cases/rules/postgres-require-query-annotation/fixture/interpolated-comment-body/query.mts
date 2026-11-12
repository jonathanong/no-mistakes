import { query, read } from '@bench/db';

declare const source: string;
declare const plan: { table: string };
declare const params: unknown[];
declare const name: string;
declare const order: string;
declare const ID: string;
declare const definition: string;

query(`/* listThingPage:${source} */\n SELECT t.id FROM ${plan.table} t WHERE t.id = $1`, params);

const statement = `/* ${name} */ SELECT t.id FROM things t ORDER BY t.id ${order} LIMIT 1`;
await read(statement, params);

read(`/* ${ID} */ ${definition}`);

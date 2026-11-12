import { read, query } from "@bench/db";
declare const name: string;
declare const source: string;
declare const head: string;
declare const close: string;
declare const suffix: string;
declare const v: string;
declare const flag: boolean;

await read(`/* ${name} */ SELECT 1`); // accepted: interpolated name
await query(`/* listPage:${source} */ SELECT id FROM t`); // accepted: literal prefix
await read(`/* ${name}:${source} */ SELECT 1`); // accepted: multiple name expressions
await read(`\x2f* ${name} *\x2f SELECT 1`); // accepted: cooked delimiters
await read(`/* literal */ SELECT ${v}`); // accepted: existing SQL interpolation
const q = `/* ${name} */ SELECT id FROM t WHERE id = $1`;
await read(q, [v]); // accepted: one local const

await read(`${head} SELECT 1`); // reported: whole head interpolated
await read(`SELECT 1 /* ${name} */`); // reported: comment is not first
await read(`/* ${name} ${close} SELECT 1`); // reported: closing delimiter interpolated
await read(`/* ${name} */ ${suffix} SELECT 1`); // reported: interpolation outside comment
await read(`/* ${''} */ SELECT 1`); // reported: known empty name
await read(`/* ${'*/'} */ SELECT 1`); // reported: known early close
await read(`/* ${flag ? '' : 'named'} */ SELECT 1`); // reported: possibly empty name
for (const item of ['', 'named']) {
  await read(`/* ${item} */ SELECT 1`); // reported: possibly empty name
}
const emptyName = '';
const emptyQuery = `/* ${emptyName} */ SELECT 1`;
await read(emptyQuery); // reported: const-bound empty name
const earlyCloseQuery = `/* ${'*/'} */ SELECT 1`;
await read(earlyCloseQuery); // reported: const-bound early close
const goodName = 'named';
const goodQuery = `/* ${goodName} */ SELECT 1`;
await read(goodQuery); // accepted: const-bound known valid name
{
  const emptyQuery = `/* shadowed */ SELECT 1`;
  await read(emptyQuery); // accepted: shadowed valid const
}
const unusedBad = `/* ${''} */ SELECT 1`;
{
  const unusedBad = `/* shadowed again */ SELECT 1`;
  await read(unusedBad); // accepted: unused outer bad const does not taint shadow
}

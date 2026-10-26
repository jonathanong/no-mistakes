import type { Executor, PoolClient } from '@example/db';
export function run(declared: Executor, omitted: PoolClient, dynamic: string) {
  declared.query('SELECT 1');
  omitted.query('SELECT 2');
  omitted['query']('SELECT 3');
  omitted[`query`]('SELECT 4');
  omitted[`\u0071uery`]('SELECT 5');
  omitted.execute('SELECT 6');
  omitted[`execute`]('SELECT 7');
  omitted[`que${dynamic}ry`]('SELECT ignored');
}

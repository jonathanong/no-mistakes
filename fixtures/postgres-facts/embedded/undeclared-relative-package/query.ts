import type { PoolClient } from './client';
import type { OutsideClient } from '../undeclared-external';
import type { DirectClient } from '@example/db';
export function run(inside: PoolClient, outside: OutsideClient, direct: DirectClient) {
  inside.query('SELECT 1');
  outside.query('SELECT 2');
  direct.query('SELECT 3');
}
// Merely importing a helper must not add a resolver lookup for diagnostics.
import { helper } from './helper';
export const text = helper('SELECT 4');

import sql from 'sql-template-strings';
import { read } from '@bench/db';
import { imported } from './external.mjs';

export function stable() { return sql`/* stable */ SELECT 1`; }
export { stable as alias };
read(stable()); // low: true
read(imported()); // low: false

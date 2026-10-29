import { read } from '@bench/db';
import sql from 'sql-template-strings';
import { sql as custom } from '@example/custom';

const untrusted = custom`/* custom */ SELECT 1`;
read(untrusted); // low: false
read(untrusted.append(dynamic)); // low: false
read(untrusted); // low: false
const trusted = sql`/* trusted */ SELECT 1`;
read(trusted.append(dynamic)); // low: true
read(trusted); // low: true
const scalar = '/* scalar */ SELECT 1';
read(scalar.append(dynamic)); // low: false
// The primitive string itself is immutable even if a custom append method runs.
read(scalar); // low: true

import { sql } from 'sql-template-strings';
import { sql as SQL } from 'sql-template-strings';
import { default as anyTag } from 'sql-template-strings';
import { sql as aliasedTag } from 'sql-template-strings';
import { helper as sQl } from 'sql-template-strings';
import type { sql as SqL } from '@other/sql';
import { type sql as sQL } from '@other/sql';
import type sqL = require('@other/sql');
// Case-sensitive import aliases do not widen conventional tag trust.
[sql`SELECT 1`, SQL`SELECT 1`, anyTag`SELECT 1`, aliasedTag`SELECT 1`, sQl`SELECT 1`, SqL`SELECT 1`, sQL`SELECT 1`, sqL`SELECT 1`];

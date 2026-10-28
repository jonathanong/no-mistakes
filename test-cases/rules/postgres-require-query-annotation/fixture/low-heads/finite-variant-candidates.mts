import sql from 'sql-template-strings';
import { read } from '@bench/db';

function fixedSuffix() { return sql` WHERE true`; }
const statement = sql`/* finite-prefix */ SELECT 1`;
statement.append(fixedSuffix());
read(statement);
read(statement.append(fixedSuffix()));
read(sql`/* direct-finite */ SELECT 1`.append(fixedSuffix()));

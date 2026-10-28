import sql from 'sql-template-strings';
import { read } from '@bench/db';
declare const custom: typeof sql;
declare const suffix: unknown;
declare function stash(value: unknown): void;

read(sql`SELECT 1`.append(suffix));
read(custom`SELECT 1`.append(suffix));
read(unbound`SELECT 1`.append(suffix));

const trusted = sql`SELECT 1`;
trusted.append(suffix);
read(trusted);
const other = custom`SELECT 1`;
other.append(suffix);
read(other);
const escaped = sql`SELECT 1`;
stash(escaped);
escaped.append(suffix);
read(escaped);

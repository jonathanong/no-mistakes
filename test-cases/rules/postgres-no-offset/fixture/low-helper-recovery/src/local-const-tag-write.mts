import SQL from 'sql-template-strings';
import { read } from '@example/db';
function otherTag(strings: TemplateStringsArray) { return strings[0]; }
// A write to the imported tag invalidates every later tagged initializer.
SQL = otherTag;
const statement = SQL`SELECT 1 OFFSET 9`;
read(statement); // RESIDUAL: the tag binding was reassigned.

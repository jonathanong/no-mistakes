import { query, pool as p, pool as db } from '@example/db';
import * as ns from '@example/db';

declare const obj: { query(sql: string): Promise<void> };

query('DROP TABLE app.users'); // direct mutation
p.query('DROP TABLE app.imported_member'); // imported member receiver advisory
obj.query('DROP TABLE app.object_member'); // unrelated local member stays outside scope
ns.query('DROP TABLE app.namespace_member'); // namespace member receiver advisory
await db.query({ text: 'DROP TABLE app.awaited_member' }); // awaited object member advisory
const sleep = 'SELECT pg_sleep(1)';
query(sleep); // one local const mutation
query('SELECT id FROM app.users'); // safe direct SQL

declare function opaqueSql(): string;
query(opaqueSql()); // residual helper
const run = query;
run('DROP TABLE app.aliased'); // residual executor alias
query('CREATE TABLE broken ('); // residual invalid SQL
declare const opaque: string;
// A complete banned statement remains certain even with an opaque next statement.
query(`CREATE TABLE app.prefix (id integer); ${opaque}`); // prefix CREATE
query(`SET session_replication_role = replica; ${opaque}`); // prefix SET
query(`SELECT pg_sleep(1); ${opaque}`); // prefix sleep
query(`SELECT 1; ${opaque}`); // safe prefix, residual tail
const constPrefix = `DROP TABLE app.const_prefix; ${opaque}`; // const prefix DROP
query(constPrefix); // const prefix call
query(`SELECT 'DROP TABLE hidden;'; ${opaque}`); // quoted semicolon stays safe
query(`SELECT 1 /* DROP TABLE hidden; */; ${opaque}`); // commented semicolon stays safe
query(`CREATE TABLE unfinished (id integer; ${opaque}`); // invalid prefix template
function offsetSql(): string { return 'SELECT id FROM app.users OFFSET 9'; }
query(offsetSql()); // mixed helper is recovered for both rules
query(`DROP\x20TABLE app.escaped; ${opaque}`); // escaped prefix stays unproved
query(`SELECT 1;
DROP TABLE app.multiline;
${opaque}`); // multiline direct prefix
query(`SELECT ${opaque}`); // no-semicolon interpolation stays unproved
let mutableSql = 'SELECT id FROM app.users';
mutableSql = opaque;
query(mutableSql); // mutable assignment stays unproved
let destructuredSql = 'SELECT id FROM app.users';
({ destructuredSql } = { destructuredSql: opaque });
query(destructuredSql); // destructuring assignment stays unproved
query('DROP\x20TABLE app.escaped_string;'); // escaped string stays unproved

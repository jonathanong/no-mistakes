import { query, sql } from "@example/db";

// Promise and iterator objects are ordinary binds even when their eventual
// return/yield values are SQL builders. Synchronous builders still splice.
async function asynchronous() { return sql`WHERE active`; }
const asyncArrow = async () => sql`WHERE active`;
const asyncExpression = async function () { return sql`WHERE active`; };
function* generator() { yield sql`WHERE active`; return sql`WHERE active`; }
const generatorExpression = function* () { return sql`WHERE active`; };
const asyncAlias = asynchronous;
query(sql`SELECT id FROM accounts WHERE payload = ${asynchronous()}`);
query(sql`SELECT id FROM accounts WHERE payload = ${asyncArrow()}`);
query(sql`SELECT id FROM accounts WHERE payload = ${asyncExpression()}`);
query(sql`SELECT id FROM accounts WHERE payload = ${generator()}`);
query(sql`SELECT id FROM accounts WHERE payload = ${generatorExpression()}`);
query(sql`SELECT id FROM accounts WHERE payload = ${asyncAlias()}`);
function synchronous() { return sql`WHERE active`; }
const syncArrow = () => sql`WHERE active`;
query(sql`SELECT id FROM accounts ${synchronous()}`);
query(sql`SELECT id FROM accounts ${syncArrow()}`);

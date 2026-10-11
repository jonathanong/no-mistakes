import sql, { SQLStatement } from "sql-template-strings";
import { query } from "@example/db";
const gate = runtimeFlag;

function destructuredParameter({ piece }: SQLStatement) { return piece; }
function twoBuilders(first: SQLStatement, second: SQLStatement) { return first; }
function uninitializedAlias(uninitialized: SQLStatement) { let alias; return uninitialized; }
function bareBuilder(bare: SQLStatement) { bare; return bare; }
function noReturn(missing: SQLStatement) { return; }
function returnedOpaque(opaque: SQLStatement) { return opaque.append(runtimeTail); }
function destructuredAlias(local: SQLStatement) { const { piece } = local; return piece; }
function multipleArguments(multiple: SQLStatement) { return multiple.append(sql` WHERE active`, sql` LIMIT 1`); }
function optionalReceiver(optional: SQLStatement) { return optional?.append(sql` WHERE active`); }
function optionalAppend(optionalCall: SQLStatement) { return optionalCall.append?.(sql` WHERE active`); }
function otherMember(member: SQLStatement) { return member.other(sql` WHERE active`); }
function returnedReceiver(receiver: SQLStatement) { return getBuilder().append(sql` WHERE active`); }
function unrelatedReceiver(unrelated: SQLStatement) { return other.append(sql` WHERE active`); }
function spreadFragment(spread: SQLStatement) { return spread.append(...pieces); }
function unknownPlaceholder(placeholder: SQLStatement) { return placeholder.append(sql` WHERE owner = ${unknownOwner}`); }
function withOwner(ownerBuilder: SQLStatement, owner: string) { return ownerBuilder.append(sql` WHERE owner = ${owner}`); }
function emptyStatement(empty: SQLStatement) { ; empty.append(sql` FROM users`); return empty; }

query(gate ? destructuredParameter(sql`SELECT id`) : sql`SELECT 1`);
query(gate ? twoBuilders(sql`SELECT id`, sql`SELECT id`) : sql`SELECT 2`);
query(gate ? uninitializedAlias(sql`SELECT id`) : sql`SELECT 3`);
query(gate ? bareBuilder(sql`SELECT id`) : sql`SELECT 4`);
query(gate ? noReturn(sql`SELECT id`) : sql`SELECT 5`);
query(gate ? returnedOpaque(sql`SELECT id`) : sql`SELECT 6`);
query(gate ? destructuredAlias(sql`SELECT id`) : sql`SELECT 7`);
query(gate ? multipleArguments(sql`SELECT id`) : sql`SELECT 8`);
query(gate ? optionalReceiver(sql`SELECT id`) : sql`SELECT 9`);
query(gate ? optionalAppend(sql`SELECT id`) : sql`SELECT 10`);
query(gate ? otherMember(sql`SELECT id`) : sql`SELECT 11`);
query(gate ? returnedReceiver(sql`SELECT id`) : sql`SELECT 12`);
query(gate ? unrelatedReceiver(sql`SELECT id`) : sql`SELECT 13`);
query(gate ? spreadFragment(sql`SELECT id`) : sql`SELECT 14`);
query(gate ? unknownPlaceholder(sql`SELECT id`) : sql`SELECT 15`);
const existing = sql`SELECT id FROM users`;
query(gate ? withOwner(existing, ...labels) : sql`SELECT 16`);
const stringLocal = "SELECT id FROM users";
query(withOwner(stringLocal, "literal"));
const dynamicBuilder = sql`SELECT id ${sql.raw(runtimeClause)}`;
query(withOwner(dynamicBuilder, "literal"));
query(withOwner(getBuilder(), "literal"));
query(gate ? emptyStatement(sql`SELECT id`) : sql`SELECT 17`);

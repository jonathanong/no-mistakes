import { query, sql } from "@example/db";

const inner = sql`SELECT 1`; // missing: the finite guarded version exposes this token.
// Plain strings, booleans, numbers, and objects stay binds, even through aliases.
query(sql`${String.raw`SELECT 1`}`); // missing
const raw = String.raw`SELECT 1`;
query(sql`${raw}`); // missing
query(sql`${inner === inner}`); // missing
const comparison = inner === inner;
query(sql`${comparison}`); // missing
query(sql`${inner ? 1 : 2}`); // missing
const conditional = inner ? 1 : 2;
query(sql`${conditional}`); // missing
query(sql`${{ inner }}`); // missing
const object = { inner };
query(sql`${object}`); // missing

// Runtime candidates and arrays stay opaque; the static guard has finite versions.
query(sql`${cond ? inner : 1}`); // unknown
const possible = cond ? inner : 1;
query(sql`${possible}`); // unknown
query(sql`${cond && inner}`); // missing: the false guard executes empty SQL.
const logical = cond || inner;
query(sql`${logical}`); // unknown
query(sql`${[inner]}`); // unknown
const array = [inner];
query(sql`${array}`); // unknown
query(sql`${[inner][key]}`); // unknown
const indexed = [inner][key];
query(sql`${indexed}`); // unknown
query(sql`${object.inner}`); // unknown
const property = object.inner;
query(sql`${property}`); // unknown
query(sql`${object[key]}`); // unknown
const dynamicProperty = object[key];
query(sql`${dynamicProperty}`); // unknown
query(sql`${{ 0: inner }[0]}`); // unknown
query(sql`${{ nested: { inner } }.nested.inner}`); // unknown
query(sql`${{ ...object }.inner}`); // unknown
const objectChoice = cond ? { inner } : { other: 1 };
query(sql`${objectChoice}`); // missing
query(sql`${objectChoice.inner}`); // unknown
const chosenProperty = objectChoice[key];
query(sql`${chosenProperty}`); // unknown
// A builder used only as a key never turns scalar property values into fragments.
query(sql`${{ [inner]: 1 }.value}`); // missing
const keyedScalar = { [inner]: 1 };
query(sql`${keyedScalar[key]}`); // missing
query(sql`${[1][inner]}`); // missing
query(sql`${{ [inner]: inner }[key]}`); // unknown

query(sql`/* scalar binds */ ${String.raw`SELECT 1`} ${comparison} ${conditional} ${object}`);

// Forwarding a comparison through a helper retains its ordinary bind result.
function identity(value) { return value; }
const forwardedComparison = identity(comparison);
query(sql`${forwardedComparison}`); // missing

// A possibly deleted argument property cannot establish an annotation.
function possiblyDeleted() {
  if (cond) delete arguments[0];
  query(arguments[0].inner); // unknown
}
possiblyDeleted({ inner: sql`SELECT 9` });

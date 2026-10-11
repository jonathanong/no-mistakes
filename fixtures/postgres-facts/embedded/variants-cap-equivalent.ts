import { query, sql } from "@example/db";
const shared = sql`/* same */`;
// Each condition chooses the same physical fragment. These are one version,
// even though five independent binary decisions have 32 execution paths.
query(sql`SELECT 1 ${a ? shared : shared} ${b ? shared : shared} ${c ? shared : shared} ${d ? shared : shared} ${e ? shared : shared}`);

const flag = opaque();
const prefix = flag ? sql`/* yes */` : sql`/* no */`;
const suffix = flag ? sql`/* true */` : sql`/* false */`;
// Removing irrelevant choices must keep choices that correlate real fragments.
query(sql`SELECT 2 ${a ? shared : shared} ${b ? shared : shared} ${c ? shared : shared} ${d ? shared : shared} ${e ? shared : shared} ${prefix} ${suffix}`);

const left = flag ? sql`/* left */` : shared;
const right = flag ? shared : sql`/* right */`;
// A fragment appearing on opposite paths is not an unconstrained alternative.
query(sql`SELECT 3 ${left} ${right}`);

const first = gateA ? true : false;
const second = gateB ? true : false;
const third = gateC ? true : false;
const fourth = gateD ? true : false;
const fifth = gateE ? true : false;
// Recovered boolean guards have their own path choices. None affect this SQL.
query(sql`SELECT 4 ${first ? shared : shared} ${second ? shared : shared} ${third ? shared : shared} ${fourth ? shared : shared} ${fifth ? shared : shared}`);

// The fast path preserves choices already owned by the selected fragment.
const selected = flag ? left : right;
query(sql`SELECT 5 ${first ? selected : selected} ${second ? selected : selected} ${third ? selected : selected} ${fourth ? selected : selected} ${fifth ? selected : selected} ${suffix}`);

// Builder wrappers recover the same physical SQL without identical arm syntax.
query(sql`SELECT 6 ${flag ? sql(shared) : sql(shared)}`);

const changed = sql`SELECT 7`;
// Equal arm identifiers cannot hide a mutation while evaluating the guard.
query((changed.append(" LIMIT 1"), flag) ? changed : changed);

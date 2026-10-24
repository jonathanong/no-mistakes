import { query } from "@example/db";
import { imported } from "./unknown";
let mutable = "BEGIN";
const fromMutable = mutable;
query(fromMutable);
const changed = "BEGIN";
const changedAlias = changed;
changedAlias = "SELECT 1";
query(changedAlias);
query(before);
const before = later;
const later = "BEGIN";
const cycleA = cycleB;
const cycleB = cycleA;
query(cycleA);
const interpolated = `BEGIN ${value}`;
const interpolationAlias = interpolated;
query(interpolationAlias);
const tagged = sql`BEGIN`;
const tagAlias = tagged;
query(tagAlias);
const concatenated = "BE" + "GIN";
const concatAlias = concatenated;
query(concatAlias);
const fromImport = imported;
query(fromImport);
const container = { sql: "BEGIN" };
const fromContainer = container.sql;
query(fromContainer);
const { sql: destructured } = container;
const destructureAlias = destructured;
query(destructureAlias);
const helper = () => "BEGIN";
const returned = helper();
const returnAlias = returned;
query(returnAlias);
const outer = "BEGIN";
const outerAlias = outer;
function crossing() { query(outerAlias); }
{ query(outerAlias); }
function shadow(outer: string) { const a = outer; query(a); }
const select = "SELECT 1";
const selectAlias = select;
query(selectAlias);
// An extra reference in another scope keeps the entire new chain unknown.
const escaped = "BEGIN";
const escapedAlias = escaped;
function capture() { return escapedAlias; }
query(escapedAlias);
const missing = unknown;
query(missing);
const number = 12;
const numberAlias = number;
query(numberAlias);
try { throw "BEGIN"; } catch (caught) { const a = caught; query(a); }
query();

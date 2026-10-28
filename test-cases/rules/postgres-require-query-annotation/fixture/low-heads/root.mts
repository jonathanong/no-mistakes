import { read, sql } from "@bench/db";
import { imported } from "./imported.mjs";
const named = "/* local */ SELECT 1";
const unnamed = "SELECT 1";
const alias = named;
let mutable = "/* mutable */ SELECT 1";
var oldStyle = "/* var */ SELECT 1";
const { destructured } = object;
declare const declared: string;
read("/* direct */ SELECT 1"); // low: true
read(`/* template */ SELECT ${value}`); // low: true
read("/* plus */ " + value); // low: true
read("/* left spine */ " + value + tail); // low: true
read(`/* template plus */ ${value}` + tail); // low: true
read("/* tail choice */ " + (flag ? left : right)); // low: true
// Nested calls retain distinct original spans even on the same physical line.
read("/* outer */ " + read("SELECT 2")); // low: true,false
read(named); // low: true
read(named as string); // low: true
read((named satisfies string)!); // low: true
read(unnamed); // low: false
read(alias); // low: false
read(mutable); // low: false
read(oldStyle); // low: false
read(destructured); // low: false
read(declared); // low: false
read(imported); // low: false
read(named + tail); // low: false
read(helper("/* helper */ SELECT 1")); // low: false
read(sql`/* tag */ SELECT 1`); // low: true
read(String.raw`/* raw */ SELECT 1`); // low: true
read(flag ? "/* first */ SELECT 1" : "/* second */ SELECT 2"); // low: true
read(`/* ${value} */ SELECT 1`); // low: false
read("/*" + " split */ SELECT 1"); // low: false
read(value + "/* right */ SELECT 1"); // low: false
read("/* */ /* later */ SELECT 1"); // low: false
read("/**/ SELECT 1"); // low: false
read("/*   */ SELECT 1"); // low: false
read("/* open"); // low: false
read("BEGIN"); // low: true
read("COMMIT;"); // low: true
read("ROLLBACK TO SAVEPOINT point"); // low: true
read("SELECT 1"); // low: false
read(""); // low: false
read(/* JavaScript comments are not SQL text. */ value); // low: false
read("\u002f\u002a escaped \u002a\u002f SELECT 1"); // low: true
read(`\u002f* cooked quasi */ SELECT ${value}`); // low: true
read(`
  /* multiline
     name */ SELECT ${value}`); // low: true
read(); // low: false
read(...parts); // low: false
read((named, unnamed)); // low: false
read(named.append("/* appended */ SELECT 1")); // low: false
const prefix = "/* const plus */ " + value;
const templated = `/* const template */ ${value}`;
const constAlias = prefix;
read(prefix); // low: true
read(templated); // low: true
read(constAlias); // low: false
function captured() {
  read(named);
} // low: true
function captureLater() {
  read(later);
} // low: true
const later = "/* declared after the closure, before invocation */ SELECT 1";
captureLater();
function shadow(named: string) {
  read(named);
} // low: false
{
  let named = "/* shadow let */ SELECT 1";
  read(named);
} // low: false
{
  const named = "/* shadow const */ SELECT 1";
  read(named);
} // low: true
interface Merged {
  one: number;
}
interface Merged {
  two: string;
}
// An unrelated merged-interface diagnostic must not poison the const proof.
read(named); // low: true
read(named); // low: true
read(unnamed); // low: false

import sql from "sql-template-strings";
import { query } from "@example/db";
const gate = runtimeFlag;
let shorthand = gate ? "SELECT 1" : "SELECT 2";
({ shorthand } = { shorthand: "SELECT 1 OFFSET 1" });
query(shorthand); // opaque
let renamed = gate ? "SELECT 3" : "SELECT 4";
({ statement: renamed } = { statement: "SELECT 3 OFFSET 2" });
query(renamed); // opaque
let array = gate ? "SELECT 5" : "SELECT 6";
[, array] = [0, "SELECT 5 OFFSET 3"];
query(array); // opaque
let nested = gate ? "SELECT 7" : "SELECT 8";
({ statements: [{ statement: nested }] } = { statements: [{ statement: "SELECT 7 OFFSET 4" }] });
query(nested); // opaque
let objectDefault = gate ? "SELECT 9" : "SELECT 10";
({ statement: objectDefault = "SELECT 9 OFFSET 5" } = {});
query(objectDefault); // opaque
let arrayDefault = gate ? "SELECT 11" : "SELECT 12";
[arrayDefault = "SELECT 11 OFFSET 6"] = [];
query(arrayDefault); // opaque
let shorthandDefault = gate ? "SELECT 13" : "SELECT 14";
({ shorthandDefault = "SELECT 13 OFFSET 7" } = {});
query(shorthandDefault); // opaque
let arrayRest = gate ? "SELECT 15" : "SELECT 16";
[...arrayRest] = ["SELECT 15 OFFSET 8"];
query(arrayRest); // opaque
let objectRest = gate ? "SELECT 17" : "SELECT 18";
({ ...objectRest } = { statement: "SELECT 17 OFFSET 9" });
query(objectRest); // opaque
let computed = gate ? "SELECT 19" : "SELECT 20";
const key = "statement";
({ [key]: computed } = { statement: "SELECT 19 OFFSET 10" });
query(computed); // opaque
/* Unverified property writes invalidate builders, including apparent metadata. */
const receiver = gate ? sql`SELECT 21` : sql`SELECT 22`;
const untouched = gate ? sql`SELECT 21` : sql`SELECT 22`;
query(untouched); // variants: SELECT 21 | SELECT 22
({ statement: receiver.extra } = { statement: "unrelated" });
[receiver["extra"]] = ["also unrelated"];
query(receiver); // opaque
query(untouched); // variants: SELECT 21 | SELECT 22
let scalar = 0;
({ scalar } = { scalar: 1 });
query(receiver); // opaque
let shadowed = gate ? "SELECT 23" : "SELECT 24";
{
  let shadowed = gate ? "SELECT 25" : "SELECT 26";
  ({ shadowed } = { shadowed: "SELECT 25 OFFSET 11" });
  query(shadowed); // opaque
}
query(shadowed); // variants: SELECT 23 | SELECT 24
let replaced = gate ? sql`SELECT 36` : sql`SELECT 37`;
const oldValue = replaced;
({ replaced } = { replaced: sql`SELECT 36 OFFSET 15` });
query(replaced); // opaque
query(oldValue); // variants: SELECT 36 | SELECT 37
let copied = gate ? "SELECT 38" : "SELECT 39";
const oldPrimitive = copied;
({ copied } = { copied: "SELECT 38 OFFSET 16" });
query(oldPrimitive); // variants: SELECT 38 | SELECT 39
query(copied); // opaque
/* RHS, default initializers and computed keys execute before their target is written. */
let rhs = gate ? "SELECT 40" : "SELECT 41";
({ rhs } = { rhs: (query(rhs), "SELECT 40 OFFSET 17") }); // variants: SELECT 40 | SELECT 41
query(rhs); // opaque
let defaulted = gate ? "SELECT 42" : "SELECT 43";
({ defaulted = (query(defaulted), "SELECT 42 OFFSET 18") } = {}); // variants: SELECT 42 | SELECT 43
query(defaulted); // opaque
let keyOrder = gate ? "SELECT 44" : "SELECT 45";
({ [(query(keyOrder), "statement")]: keyOrder } = { statement: "SELECT 44 OFFSET 19" }); // variants: SELECT 44 | SELECT 45
query(keyOrder); // opaque
/* An earlier target is already written when a later default or key executes. */
let arrayOrder = gate ? "SELECT 56" : "SELECT 57";
let laterArray;
[arrayOrder, laterArray = query(arrayOrder)] = ["SELECT 56 OFFSET 25"]; // opaque
let objectOrder = gate ? "SELECT 58" : "SELECT 59";
let laterObject;
({ first: objectOrder, second: laterObject = query(objectOrder) } = { first: "SELECT 58 OFFSET 26" }); // opaque
let computedOrder = gate ? "SELECT 60" : "SELECT 61";
({ first: computedOrder, [(query(computedOrder), "second")]: laterObject } = { first: "SELECT 60 OFFSET 27" }); // opaque
/* RHS evaluation precedes every target, even if it occurs later in source text. */
let rhsFirst = gate ? "SELECT 62" : "SELECT 63";
[
  rhsFirst,
  laterArray = query(rhsFirst), // opaque
] = (query(rhsFirst), // variants: SELECT 62 | SELECT 63
  ["SELECT 62 OFFSET 28"]);
[(receiver.extra as string) = (query(receiver), "unrelated")] = []; // opaque
[[] = (query(untouched), [])] = []; // variants: SELECT 21 | SELECT 22
[{} = (query(untouched), {})] = []; // variants: SELECT 21 | SELECT 22
let asTarget = gate ? "SELECT 46" : "SELECT 47";
[asTarget as string] = ["SELECT 46 OFFSET 20"];
query(asTarget); // opaque
let satisfiesTarget = gate ? "SELECT 48" : "SELECT 49";
[satisfiesTarget satisfies string] = ["SELECT 48 OFFSET 21"];
query(satisfiesTarget); // opaque
let nonNullTarget = gate ? "SELECT 50" : "SELECT 51";
[nonNullTarget!] = ["SELECT 50 OFFSET 22"];
query(nonNullTarget); // opaque
let assertionTarget = gate ? "SELECT 52" : "SELECT 53";
[<string>assertionTarget] = ["SELECT 52 OFFSET 23"];
query(assertionTarget); // opaque
[(receiver.extra as string)] = ["still unrelated"];
query(receiver); // opaque
shorthand = gate ? "SELECT 27" : "SELECT 28";
query(shorthand); // variants: SELECT 27 | SELECT 28
let guard = gate ? "SELECT 29" : "SELECT 30";
query(({ guard } = { guard: "SELECT 29 OFFSET 12" }) ? guard : "SELECT 31"); // opaque
let fragment = gate ? sql`SELECT 32` : sql`SELECT 33`;
/* A destructuring interpolation invalidates recovery before later fragments are read. */
query(sql`${({ fragment } = { fragment: sql`SELECT 32 OFFSET 13` }), 0} ${fragment}`); // opaque placeholders
query(gate ? "SELECT 34 OFFSET 14" : "SELECT 35"); // variants: SELECT 34 OFFSET 14 | SELECT 35
let rootWrapped = gate ? "SELECT 64" : "SELECT 65";
(rootWrapped as string) = "SELECT 64 OFFSET 29";
query(rootWrapped); // opaque
/* A continue cannot revive the pre-loop SQL after a destructuring write. */
let loopWritten = gate ? "SELECT 66" : "SELECT 67";
for (const next of runtimeStatements) {
  [loopWritten] = [next];
  continue;
}
query(loopWritten); // opaque

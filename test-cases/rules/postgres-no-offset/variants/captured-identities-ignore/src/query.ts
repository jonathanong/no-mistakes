import type { SQLStatement } from "sql-template-strings";
import { sql, query } from "@example/db";
const gate = runtimeFlag;

/* Separate conditional creations have disjoint alias groups. */
let captured = gate ? sql`1` : sql`2`;
let other = gate ? sql`SELECT 3` : sql`SELECT 4`;
query(sql`SELECT ${captured} LIMIT ${(other.append(sql` OFFSET 1`), 0)}`); // variants: SELECT 1 LIMIT sql_placeholder_1 | SELECT 2 LIMIT sql_placeholder_1
const known = sql`5`;
const independent = sql`SELECT 6`;
query(sql`SELECT ${known} LIMIT ${(independent.append(sql` OFFSET 2`), 0)}`); // variants: SELECT 5 LIMIT sql_placeholder_1

/* Mutating the captured object or an alias still invalidates its snapshot. */
let same = gate ? sql`7` : sql`8`;
query(sql`SELECT ${same} LIMIT ${(same.append(sql` OFFSET 3`), 0)}`); // opaque
const aliased = sql`9`;
const alias = aliased;
query(sql`SELECT ${aliased} LIMIT ${(alias.append(sql` OFFSET 4`), 0)}`); // opaque
let uncertain = gate ? sql`10` : sql`11`;
const uncertainAlias = uncertain;
const aliasChain = uncertainAlias;
query(sql`SELECT ${uncertain} LIMIT ${(aliasChain.append(sql` OFFSET 5`), 0)}`); // opaque

/* Exact replacement identity overrides the historical alias edge. */
const original = sql`12`;
let replacement = original;
replacement = sql`SELECT 13`;
query(sql`SELECT ${original} LIMIT ${(replacement.append(sql` OFFSET 6`), 0)}`); // variants: SELECT 12 LIMIT sql_placeholder_1

let escaped = gate ? sql`14` : sql`15`;
query(sql`SELECT ${escaped} LIMIT ${(unknownHelper(escaped), 0)}`); // opaque
let helperCapture = gate ? sql`16` : sql`17`;
const helperOther = sql`SELECT 18`;
query(sql`SELECT ${helperCapture} LIMIT ${(unknownHelper(helperOther), 0)}`); // variants: SELECT 16 LIMIT sql_placeholder_1 | SELECT 17 LIMIT sql_placeholder_1
query(sql`SELECT ${helperCapture} LIMIT ${(unknownHelper(sql`SELECT 19`), 0)}`); // variants: SELECT 16 LIMIT sql_placeholder_1 | SELECT 17 LIMIT sql_placeholder_1
function addOffset(q: SQLStatement): SQLStatement {
  q.append(sql` OFFSET 7`);
  return q;
}
const typedOther = sql`SELECT 20`;
query(sql`SELECT ${helperCapture} LIMIT ${(addOffset(typedOther), 0)}`); // variants: SELECT 16 LIMIT sql_placeholder_1 | SELECT 17 LIMIT sql_placeholder_1
query(sql`SELECT ${helperCapture} LIMIT ${(addOffset(helperCapture), 0)}`); // opaque

const mixedCapture = sql`21`;
const mixedOther = sql`SELECT 22`;
query(sql`SELECT ${mixedCapture} LIMIT ${(mixedOther.append(sql` OFFSET 8`), unknownHelper({ value: mixedCapture }), 0)}`); // opaque
let repeated = gate ? sql`23` : sql`24`;
const repeatedOther = sql`SELECT 25`;
query(sql`SELECT ${repeated}, ${repeated} LIMIT ${(repeatedOther['append'](sql` OFFSET 9`), 0)}`); // variants: SELECT 23, 23 LIMIT sql_placeholder_1 | SELECT 24, 24 LIMIT sql_placeholder_1
const first = sql`26`;
const second = sql`27`;
query(sql`SELECT ${first}, ${second} LIMIT ${(unknownHelper([second]), 0)}`); // opaque

/* A fresh argument has no relationship to the captured builder. */
let argumentOther = sql`SELECT 28`;
query(sql`SELECT ${first} LIMIT ${(unknownHelper(argumentOther = sql`SELECT 29`), 0)}`); // variants: SELECT 26 LIMIT sql_placeholder_1

/* Ordinary primitive binds keep the existing non-branching call shape. */
let primitive = gate ? "first" : "second";
query(sql`SELECT ${primitive} LIMIT ${(primitive = "SELECT 30 OFFSET 30", 0)}`); // inline: SELECT sql_placeholder_1 LIMIT sql_placeholder_2
query(sql`SELECT ${"copied"} LIMIT ${0}`); // inline: SELECT sql_placeholder_1 LIMIT sql_placeholder_2

const located = gate
  ? sql`id FROM pages WHERE id = ${branchFirst} OFFSET ${skip}`
  : sql`id FROM pages WHERE id = ${branchSecond}`;
const locationOther = sql`SELECT 31`;
query(sql`SELECT ${located} LIMIT ${(locationOther.append(sql` OFFSET 10`), pageSize)}`); // located variants: SELECT id FROM pages WHERE id = sql_placeholder_1 OFFSET sql_placeholder_2 LIMIT sql_placeholder_3 | SELECT id FROM pages WHERE id = sql_placeholder_1 LIMIT sql_placeholder_2

const suppressedBranch = gate
  // no-mistakes-disable-next-line postgres-no-offset
  ? sql`id FROM suppressed_pages WHERE active OFFSET ${skip}`
  : sql`id FROM suppressed_pages WHERE active`;
const suppressedOther = sql`SELECT 32`;
query(sql`SELECT ${suppressedBranch} LIMIT ${(suppressedOther.append(sql` OFFSET 11`), pageSize)}`); // suppressed variants: SELECT id FROM suppressed_pages WHERE active OFFSET sql_placeholder_1 LIMIT sql_placeholder_2 | SELECT id FROM suppressed_pages WHERE active LIMIT sql_placeholder_1
const suppressedCapture = sql`id FROM suppressed_users`;
const suppressedAlias = suppressedCapture;
// no-mistakes-disable-next-line postgres-no-offset
query(sql`SELECT ${suppressedCapture} LIMIT ${(unknownHelper(suppressedAlias), 0)}`); // suppressed opaque

/* Final-expression rebinding is evaluated before later object effects. */
let sequenceOther = sql`SELECT 34`;
const sequenceCapture1 = sql`33`;
query(sql`SELECT ${sequenceCapture1} LIMIT ${(sequenceOther = sequenceCapture1, sequenceOther.append(sql` OFFSET 12`), 0)}`); // opaque
let receiverOther = sql`SELECT 35`;
const sequenceCapture2 = sql`33`;
query(sql`SELECT ${sequenceCapture2} LIMIT ${((receiverOther = sequenceCapture2).append(sql` OFFSET 13`), 0)}`); // opaque
let typedAlias = sql`SELECT 36`;
const sequenceCapture3 = sql`33`;
query(sql`SELECT ${sequenceCapture3} LIMIT ${(typedAlias = sequenceCapture3, addOffset(typedAlias), 0)}`); // opaque
let typedImmediate = sql`SELECT 37`;
const sequenceCapture4 = sql`33`;
query(sql`SELECT ${sequenceCapture4} LIMIT ${(addOffset(typedImmediate = sequenceCapture4), 0)}`); // opaque
let computedAlias = sql`SELECT 38`;
const sequenceCapture5 = sql`33`;
query(sql`SELECT ${sequenceCapture5} LIMIT ${((computedAlias = sequenceCapture5)['append'](sql` OFFSET 14`), 0)}`); // opaque
let scalarAlias = 0;
const sequenceCapture6 = sql`33`;
query(sql`SELECT ${sequenceCapture6} LIMIT ${(scalarAlias = sequenceCapture6, scalarAlias.append(sql` OFFSET 15`), 0)}`); // opaque
let scalarEscape = "copied";
const sequenceCapture7 = sql`33`;
query(sql`SELECT ${sequenceCapture7} LIMIT ${(scalarEscape = sequenceCapture7, unknownHelper({ value: scalarEscape }), 0)}`); // opaque
let scalarArray = false;
const sequenceCapture8 = sql`33`;
query(sql`SELECT ${sequenceCapture8} LIMIT ${(scalarArray = sequenceCapture8, unknownHelper([...[], scalarArray]), 0)}`); // opaque
let chainFirst = null;
let chainSecond = 0;
const sequenceCapture9 = sql`33`;
query(sql`SELECT ${sequenceCapture9} LIMIT ${(chainFirst = chainSecond = sequenceCapture9, unknownHelper(chainFirst), 0)}`); // opaque
let branchAlias = sql`SELECT 39`;
const sequenceCapture10 = sql`33`;
query(sql`SELECT ${sequenceCapture10} LIMIT ${(gate ? branchAlias = sequenceCapture10 : branchAlias = sql`SELECT 40`, branchAlias.append(sql` OFFSET 16`), 0)}`); // opaque
let conditionalAlias = sql`SELECT 41`;
const sequenceCapture11 = sql`33`;
query(sql`SELECT ${sequenceCapture11} LIMIT ${(conditionalAlias = gate ? sequenceCapture11 : sql`SELECT 42`, addOffset(conditionalAlias), 0)}`); // opaque
let lazyAlias = sql`SELECT 43`;
const sequenceCapture12 = sql`33`;
query(sql`SELECT ${sequenceCapture12} LIMIT ${(gate && (lazyAlias = sequenceCapture12), unknownHelper(lazyAlias), 0)}`); // opaque
let freshAlias = sql`SELECT 44`;
const sequenceCapture13 = sql`33`;
query(sql`SELECT ${sequenceCapture13} LIMIT ${(freshAlias = sql`SELECT 45`, freshAlias.append(sql` OFFSET 17`), 0)}`); // variants: SELECT 33 LIMIT sql_placeholder_1
const separateValue = sql`SELECT 46`;
let disjointAlias = sql`SELECT 47`;
const sequenceCapture14 = sql`33`;
query(sql`SELECT ${sequenceCapture14} LIMIT ${(disjointAlias = separateValue, addOffset(disjointAlias), 0)}`); // variants: SELECT 33 LIMIT sql_placeholder_1
let uncertainValue = sql`SELECT 48`;
const sequenceCapture15 = sql`33`;
query(sql`SELECT ${sequenceCapture15} LIMIT ${(uncertainValue = unknownBuilder(), uncertainValue.append(sql` OFFSET 18`), 0)}`); // opaque
let primitiveAlias = "copied";
const sequenceCapture16 = sql`33`;
query(sql`SELECT ${sequenceCapture16} LIMIT ${(primitiveAlias = 1, unknownHelper(primitiveAlias), 0)}`); // variants: SELECT 33 LIMIT sql_placeholder_1
const sequenceCapture17 = sql`33`;
query(sql`SELECT ${sequenceCapture17} LIMIT ${(primitiveAlias = "copied", unknownHelper(primitiveAlias), 0)}`); // variants: SELECT 33 LIMIT sql_placeholder_1
const sequenceCapture18 = sql`33`;
query(sql`SELECT ${sequenceCapture18} LIMIT ${(primitiveAlias = false, unknownHelper(primitiveAlias), 0)}`); // variants: SELECT 33 LIMIT sql_placeholder_1
const sequenceCapture19 = sql`33`;
query(sql`SELECT ${sequenceCapture19} LIMIT ${(primitiveAlias = null, unknownHelper(primitiveAlias), 0)}`); // variants: SELECT 33 LIMIT sql_placeholder_1
const sequenceCapture20 = sql`33`;
query(sql`SELECT ${sequenceCapture20} LIMIT ${(primitiveAlias = 1n, unknownHelper(primitiveAlias), 0)}`); // variants: SELECT 33 LIMIT sql_placeholder_1
let wrappedAlias = sql`SELECT 49`;
const sequenceCapture21 = sql`33`;
query(sql`SELECT ${sequenceCapture21} LIMIT ${((wrappedAlias as SQLStatement) = sequenceCapture21, addOffset(wrappedAlias), 0)}`); // opaque
const sequenceCapture22 = sql`33`;
query(sql`SELECT ${sequenceCapture22} LIMIT ${((wrappedAlias satisfies SQLStatement) = sequenceCapture22, addOffset(wrappedAlias), 0)}`); // opaque
const sequenceCapture23 = sql`33`;
query(sql`SELECT ${sequenceCapture23} LIMIT ${(wrappedAlias! = sequenceCapture23, addOffset(wrappedAlias), 0)}`); // opaque
const sequenceCapture24 = sql`33`;
query(sql`SELECT ${sequenceCapture24} LIMIT ${((<SQLStatement>wrappedAlias) = sequenceCapture24, addOffset(wrappedAlias), 0)}`); // opaque
function passthrough(q: SQLStatement): SQLStatement { return q; }
let pureAlias = 0;
const sequenceCapture25 = sql`33`;
query(sql`SELECT ${sequenceCapture25} LIMIT ${(pureAlias = sequenceCapture25, passthrough(pureAlias), 0)}`); // variants: SELECT 33 LIMIT sql_placeholder_1
let conditionalFresh = sql`SELECT 50`;
const sequenceCapture26 = sql`33`;
query(sql`SELECT ${sequenceCapture26} LIMIT ${(gate ? conditionalFresh = sql`SELECT 51` : conditionalFresh = sql`SELECT 52`, unknownHelper(conditionalFresh), 0)}`); // variants: SELECT 33 LIMIT sql_placeholder_1
let lazyFresh = sql`SELECT 53`;
const sequenceCapture27 = sql`33`;
query(sql`SELECT ${sequenceCapture27} LIMIT ${(gate && (lazyFresh = sql`SELECT 54`), lazyFresh.append(sql` OFFSET 19`), 0)}`); // variants: SELECT 33 LIMIT sql_placeholder_1

/* Nested value wrappers cannot reuse identities from before their writes. */
const nestedCapture = sql`55`;
let nestedOther = sql`SELECT 56`;
let nestedTemp = sql`SELECT 57`;
query(sql`SELECT ${nestedCapture} LIMIT ${((nestedOther = (nestedTemp = nestedCapture, nestedTemp)).append(sql` OFFSET 20`), 0)}`); // opaque
const nestedTypedCapture = sql`58`;
let nestedTypedOther = sql`SELECT 59`;
let nestedTypedTemp = sql`SELECT 60`;
query(sql`SELECT ${nestedTypedCapture} LIMIT ${(addOffset(nestedTypedOther = (nestedTypedTemp = nestedTypedCapture, nestedTypedTemp)), 0)}`); // opaque
const nestedEscapeCapture = sql`61`;
let nestedEscapeTemp = sql`SELECT 62`;
query(sql`SELECT ${nestedEscapeCapture} LIMIT ${(unknownHelper((nestedEscapeTemp = nestedEscapeCapture, nestedEscapeTemp)), 0)}`); // opaque
const arrayCapture = sql`63`;
let arrayAlias = sql`SELECT 64`;
query(sql`SELECT ${arrayCapture} LIMIT ${([arrayAlias] = [arrayCapture], arrayAlias.append(sql` OFFSET 21`), 0)}`); // opaque
const objectCapture = sql`65`;
let objectAlias = sql`SELECT 66`;
query(sql`SELECT ${objectCapture} LIMIT ${({ value: objectAlias } = { value: objectCapture }, objectAlias.append(sql` OFFSET 22`), 0)}`); // opaque
const wrappedUnknownCapture = sql`67`;
let wrappedUnknown = sql`SELECT 68`;
query(sql`SELECT ${wrappedUnknownCapture} LIMIT ${(wrappedUnknown = unknownBuilder(), unknownHelper(gate ? wrappedUnknown : 0), 0)}`); // opaque
const argumentOrderCapture = sql`69`;
let firstArgument = argumentOrderCapture;
function offsetWithLabel(q: SQLStatement, label: unknown): SQLStatement { q.append(sql` OFFSET 23`); return q; }
query(sql`SELECT ${argumentOrderCapture} LIMIT ${(offsetWithLabel(firstArgument, firstArgument = sql`SELECT 70`), 0)}`); // opaque
const scalarWrapperCapture = sql`71`;
let scalarWrapper = 0;
query(sql`SELECT ${scalarWrapperCapture} LIMIT ${(unknownHelper((scalarWrapper = 1, 0)), 0)}`); // variants: SELECT 71 LIMIT sql_placeholder_1

/* Later arguments observe preceding writes; earlier values keep their snapshots. */
function addSecond(label: number, q: SQLStatement): SQLStatement { q.append(sql` OFFSET 24`); return q; }
function passthroughWithLabel(q: SQLStatement, label: unknown): SQLStatement { return q; }
const laterTypedCapture = sql`72`;
let laterTypedAlias = sql`SELECT 73`;
query(sql`SELECT ${laterTypedCapture} LIMIT ${(addSecond((laterTypedAlias = laterTypedCapture, 0), laterTypedAlias), 0)}`); // opaque
const laterUnknownCapture = sql`74`;
let laterUnknownAlias = sql`SELECT 75`;
query(sql`SELECT ${laterUnknownCapture} LIMIT ${(unknownHelper((laterUnknownAlias = laterUnknownCapture, 0), laterUnknownAlias), 0)}`); // opaque
const laterScalarCapture = sql`76`;
let laterScalarAlias = 0;
query(sql`SELECT ${laterScalarCapture} LIMIT ${(unknownHelper((laterScalarAlias = laterScalarCapture, 0), laterScalarAlias), 0)}`); // opaque
const returnedCapture = sql`77`;
let returnedOther = sql`SELECT 78`;
let returnedTemp = sql`SELECT 79`;
query(sql`SELECT ${returnedCapture} LIMIT ${(returnedOther = returnedCapture, returnedTemp = passthroughWithLabel(returnedOther, returnedOther = sql`SELECT 80`), returnedTemp.append(sql` OFFSET 25`), 0)}`); // opaque
const earlyTypedCapture = sql`81`;
let earlyTypedOther = sql`SELECT 82`;
query(sql`SELECT ${earlyTypedCapture} LIMIT ${(offsetWithLabel(earlyTypedOther, earlyTypedOther = earlyTypedCapture), 0)}`); // variants: SELECT 81 LIMIT sql_placeholder_1
const earlyUnknownCapture = sql`83`;
let earlyUnknownOther = sql`SELECT 84`;
query(sql`SELECT ${earlyUnknownCapture} LIMIT ${(unknownHelper(earlyUnknownOther, (earlyUnknownOther = earlyUnknownCapture, 0)), 0)}`); // variants: SELECT 83 LIMIT sql_placeholder_1
const receiverOrderCapture = sql`85`;
let receiverOrderOther = sql`SELECT 86`;
query(sql`SELECT ${receiverOrderCapture} LIMIT ${(receiverOrderOther.append((receiverOrderOther = receiverOrderCapture, sql` OFFSET 26`)), 0)}`); // variants: SELECT 85 LIMIT sql_placeholder_1
const laterFreshCapture = sql`87`;
let laterFreshOther = sql`SELECT 88`;
query(sql`SELECT ${laterFreshCapture} LIMIT ${(addSecond((laterFreshOther = laterFreshCapture, 0), sql`SELECT 89`), 0)}`); // variants: SELECT 87 LIMIT sql_placeholder_1
const laterFreshUnknownCapture = sql`90`;
let laterFreshUnknownOther = sql`SELECT 91`;
query(sql`SELECT ${laterFreshUnknownCapture} LIMIT ${(unknownHelper((laterFreshUnknownOther = laterFreshUnknownCapture, 0), sql`SELECT 92`), 0)}`); // variants: SELECT 90 LIMIT sql_placeholder_1
const laterScalarValueCapture = sql`93`;
let laterScalarValueOther = sql`SELECT 94`;
query(sql`SELECT ${laterScalarValueCapture} LIMIT ${(unknownHelper((laterScalarValueOther = laterScalarValueCapture, 0), 0), 0)}`); // variants: SELECT 93 LIMIT sql_placeholder_1

/* Container properties and elements also run from left to right. */
const containerArrayCapture = sql`95`;
let containerArrayAlias = sql`SELECT 96`;
query(sql`SELECT ${containerArrayCapture} LIMIT ${(unknownHelper([(containerArrayAlias = containerArrayCapture, 0), containerArrayAlias]), 0)}`); // opaque
const containerObjectCapture = sql`97`;
let containerObjectAlias = sql`SELECT 98`;
query(sql`SELECT ${containerObjectCapture} LIMIT ${(unknownHelper({ label: (containerObjectAlias = containerObjectCapture, 0), builder: containerObjectAlias }), 0)}`); // opaque

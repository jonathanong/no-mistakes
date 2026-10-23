import { read, sql } from "@example/db";

// Empty ternary suffixes are off, even when the active arm is negated.
const suffixes = sql`SELECT 10`;
suffixes.append(s0 ? " /* s0 */" : "");
suffixes.append(s1 ? " /* s1 */" : "");
suffixes.append(s2 ? " /* s2 */" : "");
suffixes.append(s3 ? " /* s3 */" : "");
suffixes.append(s4 ? " /* s4 */" : "");
suffixes.append(s5 ? " /* s5 */" : "");
suffixes.append(s6 ? " /* s6 */" : "");
suffixes.append(s7 ? " /* s7 */" : "");
suffixes.append(s8 ? " /* s8 */" : "");
suffixes.append(s9 ? " /* s9 */" : "");
suffixes.append(s10 ? " /* s10 */" : "");
suffixes.append(s11 ? " /* s11 */" : "");
suffixes.append(s12 ? " /* s12 */" : "");
suffixes.append(s13 ? " /* s13 */" : "");
suffixes.append(s14 ? " /* s14 */" : "");
suffixes.append(s15 ? " /* s15 */" : "");
suffixes.append(s16 ? " /* s16 */" : "");
suffixes.append(s17 ? " /* s17 */" : "");
suffixes.append(s18 ? " /* s18 */" : "");
suffixes.append(!lastSuffix ? " /* last */" : "");
read(suffixes);

// The inner conditional keeps its required nonempty parent fragment.
const nestedSuffix = sql`SELECT 11`;
if (n0) nestedSuffix.append(" /* n0 */");
if (n1) nestedSuffix.append(" /* n1 */");
if (n2) nestedSuffix.append(" /* n2 */");
if (n3) nestedSuffix.append(" /* n3 */");
if (n4) nestedSuffix.append(" /* n4 */");
if (n5) nestedSuffix.append(" /* n5 */");
if (n6) nestedSuffix.append(" /* n6 */");
if (n7) nestedSuffix.append(" /* n7 */");
if (n8) nestedSuffix.append(" /* n8 */");
if (n9) nestedSuffix.append(" /* n9 */");
if (n10) nestedSuffix.append(" /* n10 */");
if (n11) nestedSuffix.append(" /* n11 */");
if (n12) nestedSuffix.append(" /* n12 */");
if (n13) nestedSuffix.append(" /* n13 */");
if (n14) nestedSuffix.append(" /* n14 */");
if (n15) nestedSuffix.append(" /* n15 */");
if (n16) nestedSuffix.append(" /* n16 */");
if (n17) nestedSuffix.append(" /* n17 */");
if (n18) nestedSuffix.append(" /* n18 */");
nestedSuffix.append(!outerSuffix
  ? sql` /* outer */`.append(!innerSuffix ? sql` /* inner */` : sql``)
  : sql``);
read(nestedSuffix);


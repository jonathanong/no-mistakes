import { read, sql } from "@example/db";

// Two appends under one reversed guard are still one enabled decision.
const grouped = sql`SELECT 1`;
grouped.append(" /* common-a */");
grouped.append(" /* common-b */");
if (g0) grouped.append(" /* g0 */");
if (g1) grouped.append(" /* g1 */");
if (g2) grouped.append(" /* g2 */");
if (g3) grouped.append(" /* g3 */");
if (g4) grouped.append(" /* g4 */");
if (g5) grouped.append(" /* g5 */");
if (g6) grouped.append(" /* g6 */");
if (g7) grouped.append(" /* g7 */");
if (g8) grouped.append(" /* g8 */");
if (g9) grouped.append(" /* g9 */");
if (g10) grouped.append(" /* g10 */");
if (g11) grouped.append(" /* g11 */");
if (g12) grouped.append(" /* g12 */");
if (g13) grouped.append(" /* g13 */");
if (g14) grouped.append(" /* g14 */");
if (g15) grouped.append(" /* g15 */");
if (g16) grouped.append(" /* g16 */");
if (g17) grouped.append(" /* g17 */");
if (g18) grouped.append(" /* g18 */");
if (!last) {
  grouped.append(" /* last-a */");
  grouped.append(" /* last-b */");
}
read(grouped);

// Nested all-off paths must not crowd out distinct singleton outcomes.
const nested = sql`SELECT 2`;
if (!outer0) {
  nested.append(" /* outer0 */");
  if (!inner0) nested.append(" /* inner0 */");
}
if (outer1) {
  nested.append(" /* outer1 */");
  if (inner1) nested.append(" /* inner1 */");
}
if (outer2) {
  nested.append(" /* outer2 */");
  if (inner2) nested.append(" /* inner2 */");
}
if (outer3) {
  nested.append(" /* outer3 */");
  if (inner3) nested.append(" /* inner3 */");
}
if (outer4) {
  nested.append(" /* outer4 */");
  if (inner4) nested.append(" /* inner4 */");
}
if (outer5) {
  nested.append(" /* outer5 */");
  if (inner5) nested.append(" /* inner5 */");
}
if (outer6) {
  nested.append(" /* outer6 */");
  if (inner6) nested.append(" /* inner6 */");
}
if (outer7) {
  nested.append(" /* outer7 */");
  if (inner7) nested.append(" /* inner7 */");
}
if (outer8) {
  nested.append(" /* outer8 */");
  if (inner8) nested.append(" /* inner8 */");
}
if (outer9) {
  nested.append(" /* outer9 */");
  if (inner9) nested.append(" /* inner9 */");
}
if (!nestedLast) {
  nested.append(" /* final-a */");
  nested.append(" /* final-b */");
}
read(nested);

// A loop with two append calls is one sampled enabled decision.
const repeated = sql`SELECT 3`;
if (r0) repeated.append(" /* r0 */");
if (r1) repeated.append(" /* r1 */");
if (r2) repeated.append(" /* r2 */");
if (r3) repeated.append(" /* r3 */");
if (r4) repeated.append(" /* r4 */");
if (r5) repeated.append(" /* r5 */");
if (r6) repeated.append(" /* r6 */");
if (r7) repeated.append(" /* r7 */");
if (r8) repeated.append(" /* r8 */");
if (r9) repeated.append(" /* r9 */");
if (r10) repeated.append(" /* r10 */");
if (r11) repeated.append(" /* r11 */");
if (r12) repeated.append(" /* r12 */");
if (r13) repeated.append(" /* r13 */");
if (r14) repeated.append(" /* r14 */");
if (r15) repeated.append(" /* r15 */");
if (r16) repeated.append(" /* r16 */");
if (r17) repeated.append(" /* r17 */");
if (r18) repeated.append(" /* r18 */");
for (const item of items) {
  repeated.append(" /* loop-a */");
  repeated.append(" /* loop-b */");
}
read(repeated);

// Nested inner witness must beat arbitrary pairs from nineteen earlier guards.
const requiredParents = sql`SELECT 4`;
if (p0) requiredParents.append(" /* p0 */");
if (p1) requiredParents.append(" /* p1 */");
if (p2) requiredParents.append(" /* p2 */");
if (p3) requiredParents.append(" /* p3 */");
if (p4) requiredParents.append(" /* p4 */");
if (p5) requiredParents.append(" /* p5 */");
if (p6) requiredParents.append(" /* p6 */");
if (p7) requiredParents.append(" /* p7 */");
if (p8) requiredParents.append(" /* p8 */");
if (p9) requiredParents.append(" /* p9 */");
if (p10) requiredParents.append(" /* p10 */");
if (p11) requiredParents.append(" /* p11 */");
if (p12) requiredParents.append(" /* p12 */");
if (p13) requiredParents.append(" /* p13 */");
if (p14) requiredParents.append(" /* p14 */");
if (p15) requiredParents.append(" /* p15 */");
if (p16) requiredParents.append(" /* p16 */");
if (p17) requiredParents.append(" /* p17 */");
if (p18) requiredParents.append(" /* p18 */");
if (!parent) {
  requiredParents.append(" /* parent */");
  if (!child) requiredParents.append(" /* child */");
}
read(requiredParents);



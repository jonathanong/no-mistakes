import sql from "sql-template-strings";
type Builder = { append(text: unknown): unknown };
export function returningElse(query: Builder, access: string) {
  if (access === "owned") query.append(sql`A`);
  else if (access === "member") query.append(sql`B`);
  else return query;
}
export function missingReturn(query: Builder, access: string) {
  if (access === "owned") { query.append(sql`A`); return query; }
  if (access === "member") { query.append(sql`B`); }
  throw new Error("unknown");
}
export function conditionalThrow(query: Builder, access: string, flag: boolean) {
  if (access === "owned") { query.append(sql`A`); return query; }
  if (access === "member") { query.append(sql`B`); return query; }
  if (flag) throw new Error("unknown");
}
// A return before the final throw means the else does not always throw.
export function conditionalElse(query: Builder, access: string, flag: boolean) {
  if (access === "owned") query.append(sql`A`);
  else if (access === "member") query.append(sql`B`);
  else { if (flag) return query; throw new Error("unknown"); }
}
// Nested guards do not cover the function's whole body.
export function nested(query: Builder, access: string, flag: boolean) {
  if (flag) {
    if (access === "owned") { query.append(sql`A`); return query; }
    if (access === "member") { query.append(sql`B`); return query; }
  }
  throw new Error("unknown");
}
// Another builder in the middle prevents this group from proving a complete chain.
export function mixedChain(query: Builder, other: Builder, access: string) {
  if (access === "owned") query.append(sql`A`);
  else if (access === "other") other.append(sql`B`);
  else if (access === "member") query.append(sql`C`);
  else throw new Error("unknown");
}

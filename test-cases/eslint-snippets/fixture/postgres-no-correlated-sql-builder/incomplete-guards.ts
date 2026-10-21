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
// A zero-iteration loop bypasses this otherwise complete chain.
export function loopChain(query: Builder, access: string, flags: boolean[]) {
  for (const flag of flags) {
    if (access === "owned") query.append(sql`A`);
    else if (access === "member") query.append(sql`B`);
    else throw new Error("unknown");
  }
  return query;
}
// The arm can match without executing its append.
export function conditionalAppend(query: Builder, access: string, flag: boolean) {
  if (access === "owned") { if (flag) query.append(sql`A`); }
  else if (access === "member") query.append(sql`B`);
  else throw new Error("unknown");
  return query;
}
export function logicalAppend(query: Builder, access: string, flag: boolean) {
  if (access === "owned") { flag && query.append(sql`A`); return query; }
  if (access === "member") { query.append(sql`B`); return query; }
  throw new Error("unknown");
}
// A false outer condition bypasses the entire chain and returns an untouched builder.
export function nestedChain(query: Builder, access: string, flag: boolean) {
  if (flag)
    if (access === "owned") query.append(sql`A`);
    else if (access === "member") query.append(sql`B`);
    else throw new Error("unknown");
  return query;
}
// An earlier return can bypass a direct chain just as an outer condition can.
export function earlyExitChain(query: Builder, access: string, flag: boolean) {
  if (flag) return query;
  if (access === "owned") query.append(sql`A`);
  else if (access === "member") query.append(sql`B`);
  else throw new Error("unknown");
  return query;
}
// Optional calls also leave a path where the recognized append never executes.
export function optionalMethod(query: Partial<Builder>, access: string) {
  if (access === "owned") query.append?.(sql`A`);
  else if (access === "member") query.append?.(sql`B`);
  else throw new Error("unknown");
  return query;
}
export function optionalReceiver(query: Builder | undefined, access: string) {
  if (access === "owned") query?.append(sql`A`);
  else if (access === "member") query?.append(sql`B`);
  else throw new Error("unknown");
  return query;
}

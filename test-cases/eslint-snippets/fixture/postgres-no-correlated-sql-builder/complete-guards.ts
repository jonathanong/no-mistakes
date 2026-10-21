import sql from "sql-template-strings";
type Builder = { append(text: unknown): unknown };
// The final else excludes every unknown selector before a builder can escape.
export function chain(query: Builder, access: string) {
  if (access === "owned") query.append(sql`A`);
  else if (access === "member") query.append(sql`B`);
  else throw new Error("unknown");
  return query;
}
export function blockChain(query: Builder, access: string) {
  const label = access;
  if (access === "owned") query.append(sql`A`);
  else if (access === "member") query.append(sql`B`);
  else { console.error(label); throw new Error("unknown"); }
  return query;
}
// Each known selector returns; all remaining values reach the unconditional throw.
export function returns(query: Builder, access: string) {
  if (access === "owned") { query.append(sql`A`); return query; }
  if (access === "member") { query.append(sql`B`); return query; }
  throw new Error("unknown");
}
export const arrow = (query: Builder, access: string) => {
  if (access === "owned") return query.append(sql`A`);
  if (access === "member") return query.append(sql`B`);
  throw new Error("unknown");
};

import { read } from "@example/db";
const VARIETIES = ["painted", "plain"] as const;
type Variety = (typeof VARIETIES)[number];
type Alias = Variety;
export function literal(kind: "painted" | "plain") {
  return read(`SELECT id FROM ${kind}_widgets LIMIT 10`);
}
export function indexed(kind: Variety) {
  return read(`SELECT id FROM ${kind}_widgets LIMIT 10`);
}
export function direct(kind: (typeof VARIETIES)[number]) {
  return read(`SELECT id FROM ${kind}_widgets LIMIT 10`);
}
export function alias(kind: Alias) {
  const captured = kind;
  return read(`SELECT id FROM ${captured}_widgets LIMIT 10`);
}
export function validated(kind: string) {
  if (!VARIETIES.includes(kind)) throw new Error("Unknown variety");
  return read(`SELECT id FROM ${kind}_widgets LIMIT 10`);
}
export function someThrow(kind: string) {
  if (!VARIETIES.some(item => item === kind)) throw new Error("Unknown variety");
  return read(`SELECT id FROM ${kind}_widgets LIMIT 10`);
}
export function someReturn(kind: string) {
  if (!VARIETIES.some(item => kind === item)) return;
  return read(`SELECT id FROM ${kind}_widgets LIMIT 10`);
}
export function parenthesized(kind: ((typeof VARIETIES)[number])) {
  return read(`SELECT id FROM ${kind}_widgets LIMIT 10`);
}
const SATISFIED = ["painted", "plain"] as const satisfies readonly string[];
export function satisfied(kind: (typeof SATISFIED)[number]) {
  return read(`SELECT id FROM ${kind}_widgets LIMIT 10`);
}
export function nested(input: string) {
  {
    const table = input;
    if (!VARIETIES.some(item => item === table)) throw new Error("Unknown variety");
    return read(`SELECT id FROM ${table}_widgets LIMIT 10`);
  }
}
export function nestedShadow(table: string, input: string) {
  {
    const table = input;
    if (!VARIETIES.some(item => item === table)) throw new Error("Unknown variety");
    return read(`SELECT id FROM ${table}_widgets LIMIT 10`);
  }
}

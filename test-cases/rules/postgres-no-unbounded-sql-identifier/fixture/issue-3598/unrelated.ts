import { read } from "@example/db";
const VARIETIES = ["painted", "plain"] as const;
type Variety = (typeof VARIETIES)[number];
export function echo(VARIETIES: string) { return VARIETIES; }
export function indexed(kind: Variety) {
  return read(`SELECT id FROM ${kind}_widgets LIMIT 10`);
}

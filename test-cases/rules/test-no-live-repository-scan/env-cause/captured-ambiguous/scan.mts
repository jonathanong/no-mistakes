import { readdirSync } from "node:fs";

const captured = (unknownRoot || process.env.FIRST) || process.env.SECOND;
const first = process.env.FIRST;
const second = process.env.SECOND;
const aliasOr = first || second;
const aliasCoalesce = first ?? second;
const aliasUnknown = first || unknownRoot;
export function scan(): string[] {
  readdirSync(aliasOr);
  readdirSync(aliasCoalesce);
  readdirSync(aliasUnknown);
  return readdirSync(captured);
}

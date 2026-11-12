import { readdirSync } from "node:fs";

const multiple = (unknownRoot || process.env.FIRST) || process.env.SECOND;
readdirSync(multiple);
const mixed = unknownRoot || process.env.FIRST;
readdirSync(mixed);

const first = process.env.FIRST;
const second = process.env.SECOND;
const aliasOr = first || second;
readdirSync(aliasOr);
const aliasCoalesce = first ?? second;
readdirSync(aliasCoalesce);
const aliasUnknown = first || unknownRoot;
readdirSync(aliasUnknown);

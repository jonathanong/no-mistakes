import { readdirSync } from "node:fs";

const multiple = (unknownRoot || process.env.FIRST) || process.env.SECOND;
readdirSync(multiple);
const mixed = unknownRoot || process.env.FIRST;
readdirSync(mixed);

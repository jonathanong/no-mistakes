import { readdirSync } from "node:fs";
export function validate(root: string) { return readdirSync(root); }
export function main(args: string[]) { return validate(args[0]!); }

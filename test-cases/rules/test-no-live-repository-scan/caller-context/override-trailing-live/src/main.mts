import { writeResources } from "./writer.mts";
export function main(outputRoot) { const inputs = { outputRoot: process.cwd() }; const consumerRoot = outputRoot; const check = false; return writeResources({ outputRoot, consumerRoot, check, ...inputs }); }

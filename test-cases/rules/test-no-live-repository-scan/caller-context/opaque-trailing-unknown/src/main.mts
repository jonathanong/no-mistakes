import { writeResources } from "./writer.mts";
export function main(outputRoot, inputs) { const consumerRoot = outputRoot; const check = false; return writeResources({ outputRoot, consumerRoot, check, ...inputs }); }

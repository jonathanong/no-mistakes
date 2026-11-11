import { writeResources } from "./writer.mts";
export function main({ outputRoot, consumerRoot, check, ...inputs }) { return writeResources({ ...inputs, outputRoot, consumerRoot, check }); }

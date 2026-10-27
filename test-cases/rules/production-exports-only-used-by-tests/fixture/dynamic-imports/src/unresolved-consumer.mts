import { present } from "./absent-target.mjs";
export const usesPresent = present;
// A proven selector on a module that does not resolve stays unresolved.
export const missingModule = import("./missing.mjs").then((mod) => mod.X);
// A proven selector naming an export the module lacks stays unresolved.
export const missingMember = import("./absent-target.mjs").then((mod) => mod.absent);

import { invoke } from "./shared.mjs";
// Unsupported loop call fallbacks precede independently callable globals.
for (const item of []) { invoke("/* first */ SELECT 1"); }
for (const item of []) { invoke("SELECT 2"); }
export function later() { invoke("/* later */ SELECT 3"); }

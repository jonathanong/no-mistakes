import { read } from "@app/db";
function hidden() { read("SELECT hidden"); }
function first() { read("/* first */ SELECT 1"); }
function second() { read("SELECT 2"); }
const cycleA = cycleB;
const cycleB = [cycleA, first];
// Aliases repeat the same function start; the cycle must unwind across yields.
export const registry = { first, again: first, nested: [second], cycle: cycleA };

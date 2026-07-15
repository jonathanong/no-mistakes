// Intentional re-export cycle (cycle-a <-> cycle-b) to prove the traversal's
// visited-path guard terminates instead of recursing forever.
export * from "./cycle-b";

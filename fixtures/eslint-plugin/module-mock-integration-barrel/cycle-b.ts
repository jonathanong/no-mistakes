// Intentional re-export cycle (cycle-a <-> cycle-b); see cycle-a.ts.
export * from "./cycle-a";

/* no-mistakes: integration=network */
export function cycledProviderCall() {
  return "real";
}

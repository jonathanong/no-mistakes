import { readdirSync } from "node:fs";

// Captured when the module loads, before any test assigns an owned root.
const captured = process.env.CAPTURED_DIR ?? "./tmp/captured";

export function listCaptured() {
  return readdirSync(captured);
}

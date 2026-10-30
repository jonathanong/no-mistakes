import { readdirSync } from "node:fs";

const getDir = () => process.env.WRITE_CLIMB_DIR ?? "./tmp/write-climb";

// The assigned value climbs out of the temp root, so the variable is not owned.
export function listWritten() {
  return readdirSync(getDir());
}

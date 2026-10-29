import { readdirSync } from "node:fs";

const getDir = () => process.env.CHAIN_DIR ?? "./tmp/chain";

function scan() {
  return readdirSync(getDir());
}

// Two calls to the same callee reach this function twice from the scan.
export function chainA() {
  scan();
  return scan();
}

export function chainB() {
  return scan();
}

import { read } from "@example/db";
import { gated } from "./gated.mts";

// An omitted argument and an explicit `undefined` both take the default, so
// only the `requireLive` arm (line 6 of the helper) can run.
export async function omitted() {
  return read(gated("x"));
}

export async function explicitUndefined() {
  return read(gated("x", undefined));
}

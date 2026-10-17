import { read as execute } from "@app/db";
import { read, bare } from "./helper.mjs";

export function annotatedAlternatives(flag: boolean) {
  // Execute within each arm; ordinary mode conservatively rejects returned callback unions.
  return flag ? execute(read()) : flag ? execute(read()) : execute(read());
}

export function unannotatedAlternatives(flag: boolean) {
  return flag
    ? execute(bare()) // finding: missing
    : execute(bare()); // finding: missing
}

// A separate speculative entrypoint still reads the pristine module bindings.
export function pristineAfterChoice() {
  return execute(read());
}

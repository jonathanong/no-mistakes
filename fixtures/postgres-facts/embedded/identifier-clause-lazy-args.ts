import { read } from "@example/db";

function knownTable() {
  return "orders";
}

export function knownHelperHole(id: string) {
  return read(`SELECT id FROM ${knownTable()} WHERE id = $1`, [id]);
}

export function unknownHelperHole(id: string) {
  // The unknown callee must abstain before classifying this unsupported argument.
  return read(`SELECT id FROM ${missing([computeTable()])} WHERE id = $1`, [id]);
}

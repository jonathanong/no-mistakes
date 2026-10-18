import { read } from "@app/db";
const slots = (function pack(value) { return arguments; })("/* pristine */ SELECT 1");
export function aWrites() {
  slots[0] = "SELECT 1";
  read(slots[0]); // Known missing only in this independent entrypoint.
  let prefix = "/* builder */ ";
  prefix += "SELECT 1";
  read(prefix);
}
export function bReadsPristine() {
  read(slots[0]); // Must see the initial annotation after rollback.
}
export function cDeletesAndRecreates() {
  delete slots[0];
  slots[0] = "/* recreated */ SELECT 2";
  read(slots[0]);
}

import { read } from "@app/db";
const slots = ["/* array pristine */ SELECT 1"];
export function aWrites() {
  slots[0] = "SELECT 1";
  read(slots[0]);
}
export function bReadsPristine() {
  read(slots[0]); // Independent entrypoint must retain the initial annotation.
}
export function cDeletesAndRecreates() {
  delete slots[0];
  slots[0] = "/* recreated array */ SELECT 2";
  read(slots[0]);
}

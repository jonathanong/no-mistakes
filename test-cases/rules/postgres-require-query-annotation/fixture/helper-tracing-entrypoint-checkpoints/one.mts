import { read } from "@app/db";
// Keep the packer inline: a global helper would create a second entrypoint.
const slots = (function pack(value) { return arguments; })("/* initial */ SELECT 1");
export function only() {
  read(slots[0]);
  slots[0] = "SELECT 1";
  read(slots[0]); // Known missing after a real mapped-argument mutation.
}

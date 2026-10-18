import { read } from "@app/db";
// Pure array initialization avoids the conservative packer-call boundary.
const slots = ["/* array initial */ SELECT 1"];
export function only() {
  read(slots[0]);
  read("SELECT 1"); // Exact negative witness independent of getter effects.
  slots[0] = "SELECT 1";
  read(slots[0]);
}

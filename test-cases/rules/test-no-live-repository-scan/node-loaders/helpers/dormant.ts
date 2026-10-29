// Importing this helper runs module scope, not its uncalled function.
export function dormant() {
  module.require("node:fs");
}

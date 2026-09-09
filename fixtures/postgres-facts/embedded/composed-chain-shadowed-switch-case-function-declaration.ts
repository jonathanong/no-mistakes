import { query } from "@data-stores/psql";

function build() {
  return "DELETE FROM users";
}

export function run(mode: string) {
  switch (mode) {
    case "safe":
      // biome-ignore lint/correctness/noSwitchDeclarations: case-local function is the resolver fixture
      function build() {
        return "SELECT * FROM topics";
      }
      return query(build());
  }
}

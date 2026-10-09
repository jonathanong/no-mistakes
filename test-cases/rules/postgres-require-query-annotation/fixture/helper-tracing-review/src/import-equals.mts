import { write } from "@app/db";

namespace Other {
  export const sql = (strings: TemplateStringsArray) => strings[0];
}

import sql = Other.sql;

export function importedAliasShadowsDefaultTag() {
  return write(sql`/* import-equals shadow */ SELECT 1`); // unanalyzable:import-equals-shadow
}

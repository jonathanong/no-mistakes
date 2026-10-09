import sql, { type SQLStatement } from "sql-template-strings";

const build = (): SQLStatement => sql`/* defaultArrow */ SELECT id FROM orders`;
export default build;

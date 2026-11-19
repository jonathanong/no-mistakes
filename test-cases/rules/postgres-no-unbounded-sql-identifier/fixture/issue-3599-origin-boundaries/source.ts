import sql from "sql-template-strings";
export function copy(prefix: unknown) { prefix.append("fixed"); }
copy(sql`SELECT `);
export function Constructed(constructed: unknown) { constructed.append("fixed"); }
new Constructed(sql`SELECT `);
class Helpers {
  static copy(memberPrefix: unknown) { memberPrefix.append("fixed"); }
}
Helpers.copy(sql`SELECT `);

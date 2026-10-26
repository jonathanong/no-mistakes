import sql from "sql-template-strings";
import { write } from "@app/db";

export function repeatedConditionalUpdates(flag: boolean) {
  const statement = sql``;
  // Valid suffixes retain the repeated-join regression without invalid SQL.
  flag ? statement.append("SELECT 1") : statement.append("SELECT 2");
  flag ? statement.append(" /* branch 3 */") : statement.append(" /* branch 4 */");
  flag ? statement.append(" /* branch 5 */") : statement.append(" /* branch 6 */");
  flag ? statement.append(" /* branch 7 */") : statement.append(" /* branch 8 */");
  flag ? statement.append(" /* branch 9 */") : statement.append(" /* branch 10 */");
  flag ? statement.append(" /* branch 11 */") : statement.append(" /* branch 12 */");
  flag ? statement.append(" /* branch 13 */") : statement.append(" /* branch 14 */");
  flag ? statement.append(" /* branch 15 */") : statement.append(" /* branch 16 */");
  write(statement); // finding:repeated-conditional-updates
}

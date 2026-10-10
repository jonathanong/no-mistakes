import sql from "sql-template-strings";
import { write } from "@app/db";

export function repeatedConditionalUpdates(flag: boolean) {
  const statement = sql``;
  // Repeated conditional writes used to nest binding Aggregate values deeply.
  flag ? statement.append("SELECT 1") : statement.append("SELECT 2");
  flag ? statement.append(" SELECT 3") : statement.append(" SELECT 4");
  flag ? statement.append(" SELECT 5") : statement.append(" SELECT 6");
  flag ? statement.append(" SELECT 7") : statement.append(" SELECT 8");
  flag ? statement.append(" SELECT 9") : statement.append(" SELECT 10");
  flag ? statement.append(" SELECT 11") : statement.append(" SELECT 12");
  flag ? statement.append(" SELECT 13") : statement.append(" SELECT 14");
  flag ? statement.append(" SELECT 15") : statement.append(" SELECT 16");
  write(statement); // unanalyzable:repeated-conditional-updates
}

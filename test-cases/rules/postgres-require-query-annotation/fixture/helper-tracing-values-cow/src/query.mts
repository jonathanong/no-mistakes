import sql from "sql-template-strings";
import { write } from "@app/db";

export function orderedContainerSnapshot(flag: boolean) {
  const statements = [sql`/* safe */ SELECT 1`, sql`SELECT 2`];
  // Both indexed reads must remain unproven after the branch snapshots share this container.
  flag ? statements[0].append(" WHERE TRUE") : statements[1].append(" WHERE TRUE");
  write(statements[0]);
  write(statements[1]); // finding:second-unproven-statement
}

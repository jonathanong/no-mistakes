import sql from "sql-template-strings";
import { read } from "@example/db";
import { ownerVisible } from "@fixture/visibility";
import { ownerVisible as fromSubpath } from "@fixture/visibility/owner";
import { missingHelper } from "@fixture/not-in-the-workspace";

// A pnpm workspace package name resolves through the workspace map.
export async function viaPackageEntry(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisible("l.parent_id", "l.account_id"));
  return read(query);
}

// So does a subpath export of the same package.
export async function viaSubpath(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(fromSubpath("l.parent_id", "l.account_id"));
  return read(query);
}

// A package the workspace does not declare cannot be followed.
export async function viaUnknownPackage(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(missingHelper("l.parent_id"));
  return read(query);
}

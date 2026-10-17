import sql from "sql-template-strings";
import { read } from "@example/db";
import * as namespaced from "./visibility-sql.mts";
import { missing } from "./does-not-exist.mts";
import { fromPackage } from "some-external-package";
import { ownerVisibleSql } from "./visibility-sql.mts";
import { declaredLater } from "./visibility-sql.mts";

// None of these link to a SQL helper the resolver can prove, so each query
// stays unanalyzable: a missing module, an external package, a namespace
// member call, an export the module does not provide, and a CommonJS require.
export async function viaMissingModule(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(missing());
  return read(query);
}

export async function viaPackage(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(fromPackage());
  return read(query);
}

export async function viaNamespaceMember(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(namespaced.ownerVisibleSql("l.parent_id", "l.account_id"));
  return read(query);
}

export async function viaMissingExport(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(declaredLater());
  return read(query);
}

export async function viaRequire(id: string) {
  const { ownerVisibleSql: required } = require("./visibility-sql.mts");
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(required("l.parent_id", "l.account_id"));
  return read(query);
}

export async function viaDynamicImport(id: string) {
  const loaded = await import("./visibility-sql.mts");
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(loaded.ownerVisibleSql("l.parent_id", "l.account_id"));
  return read(query);
}

export async function viaLocalShadow(id: string, ownerVisibleSql: (a: string, b: string) => string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(ownerVisibleSql("l.parent_id", "l.account_id"));
  return read(query);
}

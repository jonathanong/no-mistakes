import sql from "sql-template-strings";
import db from "@example/db";
import query = require("@example/db");

type Builder = { append(text: unknown): unknown };

export function onceMember(kind: string) {
  if (kind === "a") db.query.append(sql`select 1`);
}

export function memberImport(kind: string) {
  if (kind === "a") db.query.append(sql`select 1`);
  if (kind === "b") db.query.append(sql`select 2`);
}

export function requiredCall(kind: string) {
  if (kind === "a") require("@example/db").append(sql`select 1`);
  if (kind === "b") require("@example/db").append(sql`select 2`);
}

export function importEqualsMember(kind: string) {
  if (kind === "a") query.builder.append(sql`select 1`);
  if (kind === "b") query.builder.append(sql`select 2`);
}

export function parameter(query: Builder, kind: string) {
  if (kind === "a") query.append(sql`select 1`);
  if (kind === "b") query.append(sql`select 2`);
}

export function importEqualsIdentifier(kind: string) {
  if (kind === "a") query.append(sql`select 1`);
  if (kind === "b") query.append(sql`select 2`);
}

export function suppressed(kind: string) {
  // no-mistakes-disable-next-line postgres-no-correlated-sql-builder -- reviewed receiver
  if (kind === "a") db.query.append(sql`select 1`);
  if (kind === "b") db.query.append(sql`select 2`);
}

export function unresolvedIdentifier(kind: string) {
  if (kind === "a") missingBuilder.append(sql`select 1`);
  if (kind === "b") missingBuilder.append(sql`select 2`);
}

export function nestedMember(kind: string) {
  if (kind === "a") {
    if (kind === "b") db.query.append(sql`select 1`);
  }
  if (kind === "c") db.query.append(sql`select 2`);
}

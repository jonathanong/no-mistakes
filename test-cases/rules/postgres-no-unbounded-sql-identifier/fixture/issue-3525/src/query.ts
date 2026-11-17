import sql from "sql-template-strings";
import { read } from "@example/db";

export function missing(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `/* alias */ ${q.text}`;
  return read(statement);
}

export function matched(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `/* alias */ ${q.text}`;
  return read(statement, q.values);
}

export function wrongOwner(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const other = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `/* alias */ ${q.text}`;
  return read(statement, other.values);
}

export function chainedMissing(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `/* alias */ ${q.text}`;
  const alias = statement;
  return read(alias);
}

export function chainedMatched(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `/* alias */ ${q.text}`;
  const alias = statement;
  return read(alias, q.values);
}

export function bindFree() {
  const q = sql`SELECT * FROM items`;
  const statement = `/* alias */ ${q.text}`;
  return read(statement);
}

export function direct(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  return read(`/* direct */ ${q.text}`, q.values);
}

export function constantBind() {
  const q = sql`SELECT * FROM items WHERE id = ${42}`;
  const statement = `/* alias */ ${q.text}`;
  return read(statement);
}

export function shadowedValues(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `/* alias */ ${q.text}`;
  {
    const q = sql`SELECT * FROM items WHERE id = ${id}`;
    return read(statement, q.values);
  }
}

export function bindAlias(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `/* alias */ ${q.text}`;
  const values = q.values;
  return read(statement, values);
}

export function optionsAfterValues(id: string) {
  const q = sql`SELECT * FROM items WHERE id = ${id}`;
  const statement = `/* alias */ ${q.text}`;
  return read(statement, q.values, { label: "example" });
}

export function rawIdentifier(table: string) {
  return read(`SELECT * FROM ${table}`);
}

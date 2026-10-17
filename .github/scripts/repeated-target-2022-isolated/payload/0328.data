import { query } from "@example/db";

const across = "SELECT id FROM posts" +
  " OFFSET 1";
export function acrossCall() {
  return query(across);
}

const nested = "SELECT " + ("id FROM posts" +
  " OFFSET 2");
export function nestedCall() {
  return query(nested);
}

const templated = `SELECT id FROM posts` +
  " OFFSET 3";
export function templatedCall() {
  return query(templated);
}

const raw = String.raw`SELECT id FROM posts` +
  " OFFSET 4";
export function rawCall() {
  return query(raw);
}

const dynamic = missing + " OFFSET 5";
export function dynamicCall() {
  return query(dynamic);
}

const interpolated = `SELECT ${1}` + " OFFSET 6";
export function interpolatedCall() {
  return query(interpolated);
}

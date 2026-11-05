import { query } from "@example/db";

type Options = { hashes: boolean; lower: boolean };

export function ordered(options: Options) {
  const hashesPredicate = options.hashes
    ? ` AND orders.content_sha = ANY(SELECT decode(hash, 'hex') FROM unnest($${options.lower ? 4 : 3}::text[]) hash)`
    : "";
  return query(`SELECT * FROM orders WHERE id = ANY($1)${hashesPredicate} ORDER BY id FOR UPDATE`);
}

export function nonKeyOrder(options: Options) {
  const hashesPredicate = options.hashes
    ? ` AND orders.content_sha = ANY(SELECT decode(hash, 'hex') FROM unnest($${options.lower ? 4 : 3}::text[]) hash)`
    : "";
  return query(`SELECT * FROM orders WHERE id = ANY($1)${hashesPredicate} ORDER BY created_at FOR UPDATE`);
}

export function orderedStrings(options: Options) {
  const hashesPredicate = options.hashes
    ? ` AND orders.content_sha = ANY(SELECT decode(hash, 'hex') FROM unnest($${options.lower ? "4" : "3"}::text[]) hash)`
    : "";
  return query(`SELECT * FROM orders WHERE id = ANY($1)${hashesPredicate} ORDER BY id FOR UPDATE`);
}

export function nonKeyOrderStrings(options: Options) {
  const hashesPredicate = options.hashes
    ? ` AND orders.content_sha = ANY(SELECT decode(hash, 'hex') FROM unnest($${options.lower ? "4" : "3"}::text[]) hash)`
    : "";
  return query(`SELECT * FROM orders WHERE id = ANY($1)${hashesPredicate} ORDER BY created_at FOR UPDATE`);
}

export function malformed() {
  return query(`SELECT * FROM orders WHERE id = ANY($1 FOR UPDATE`);
}

export function dynamic(statement: string) {
  return query(statement);
}

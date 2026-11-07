import { query } from "@example/db";
import { sql } from "@example/sql";
import { sql as untrusted } from "@example/other";

export function open(value: string, n: number) {
  const fragment = untrusted`id FROM ${value}`;
  query(`SELECT ${fragment.text}`);
  const statements = [`id = $${value}`];
  const where = statements.join(" AND ");
  query(`SELECT id FROM records WHERE ${where}`);
  const changed = [`id = $${n}`];
  changed.push(value);
  const mutated = changed.join(" AND ");
  query(`SELECT id FROM records WHERE ${mutated}`);
  const altered = sql`id FROM records WHERE id = ${value}`;
  altered.append(value);
  query(`SELECT ${altered.text}`);
  const literal = ["users", "posts"];
  const dynamicSeparator = literal.join(value);
  query(`SELECT id FROM ${dynamicSeparator}`);
}

export function shadow(sql: (value: TemplateStringsArray) => unknown, value: string) {
  const fragment = sql`id FROM ${value}`;
  query(`SELECT ${fragment.text}`);
}

export function nestedMutation(value: string) {
  const nestedPush = sql`id FROM records`;
  nestedPush.strings.push(value);
  query(`SELECT ${nestedPush.text}`);
  const nestedSplice = sql`id FROM records`;
  nestedSplice.strings.splice(0, 1, value);
  query(`SELECT ${nestedSplice.text}`);
  const nestedWrite = sql`id FROM records`;
  nestedWrite.strings[0] = value;
  query(`SELECT ${nestedWrite.text}`);
  const nestedAlias = sql`id FROM records`;
  const strings = nestedAlias.strings;
  strings[0] = value;
  query(`SELECT ${nestedAlias.text}`);
  const nestedComputed = sql`id FROM records`;
  nestedComputed["strings"].push(value);
  query(`SELECT ${nestedComputed.text}`);
}

import { query } from "@example/db";

const re = /`-- no-mistakes-bound postgres-bounded-statements rows<=1: in regex/;
const quote = /['`"]/;
const escaped = /\'/;
const continued =
  "\
`-- no-mistakes-bound postgres-bounded-statements rows<=1: continued";

const braces = `SELECT ${"{"}
-- no-mistakes-bound postgres-bounded-statements rows<=9: braces
`;
const early = `SELECT ${"} -- no-mistakes-bound postgres-bounded-statements rows<=1: early"}`;

export function orders(): Promise<unknown> {
  void re;
  void quote;
  void escaped;
  void continued;
  void braces;
  void early;
  return query("SELECT id FROM orders");
}

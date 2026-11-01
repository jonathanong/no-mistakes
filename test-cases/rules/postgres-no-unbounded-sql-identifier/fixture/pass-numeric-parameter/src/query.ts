import { query } from "@example/db";

export function shard(n: number) {
  return query(`SELECT id FROM events_${n}`);
}

export function shardPair(a: number, b: number) {
  return query(`SELECT id FROM events_${a}_${b}`);
}
